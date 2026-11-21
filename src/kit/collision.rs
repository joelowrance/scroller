//! Axis-aligned hitboxes on collision layers.
//!
//! A [`Hitbox`] is sized and placed on its own, separate from the sprite, so a
//! Hero can have a small forgiving hitbox inside a larger sprite. Only the
//! layer pairs in [`PAIRS`] are tested; every overlap found is written as a
//! [`Contact`], which the rest of the kit turns into damage, score and lost
//! lives.

use bevy::prelude::*;

/// Which group an entity collides as.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    Hero,
    HeroShot,
    Enemy,
    EnemyShot,
    Pickup,
    Terrain,
    /// Stops enemy shots and nothing else (a Hero's drone or shield).
    ShotBlocker,
    /// A target on the ground below the Hero (Cabinet 2): Hero shots and
    /// rockets hit it, but it never touches the Hero.
    Ground,
    /// A Hero shot that only hits [`Layer::Ground`] (Cabinet 2's rockets).
    Rocket,
}

/// The layer pairs that interact. In each [`Contact`], `a` is on the first
/// layer of its pair and `b` on the second.
pub const PAIRS: [(Layer, Layer); 10] = [
    (Layer::HeroShot, Layer::Enemy),
    (Layer::EnemyShot, Layer::Hero),
    (Layer::Enemy, Layer::Hero),
    (Layer::Pickup, Layer::Hero),
    (Layer::Terrain, Layer::Hero),
    (Layer::Terrain, Layer::HeroShot),
    (Layer::Terrain, Layer::EnemyShot),
    (Layer::EnemyShot, Layer::ShotBlocker),
    (Layer::HeroShot, Layer::Ground),
    (Layer::Rocket, Layer::Ground),
];

/// Whether two layers interact at all, in either order.
pub fn interacts(a: Layer, b: Layer) -> bool {
    PAIRS
        .iter()
        .any(|&(x, y)| (x, y) == (a, b) || (x, y) == (b, a))
}

/// A box relative to the entity's position, independent of its sprite.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Hitbox {
    pub size: Vec2,
    pub offset: Vec2,
}

impl Hitbox {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: Vec2::new(width, height),
            offset: Vec2::ZERO,
        }
    }

    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vec2::new(x, y);
        self
    }

    /// The box in world space for an entity at `at`.
    pub fn aabb(&self, at: Vec2) -> Aabb {
        let centre = at + self.offset;
        Aabb {
            min: centre - self.size / 2.0,
            max: centre + self.size / 2.0,
        }
    }
}

/// A second box used only against [`Layer::Terrain`], so a ship can have a
/// tiny hitbox for bullets and a wider one that keeps its wings out of walls.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TerrainHitbox(pub Hitbox);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    /// True when the boxes share area. Boxes that only touch edges don't.
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && other.min.x < self.max.x
            && self.min.y < other.max.y
            && other.min.y < self.max.y
    }
}

/// Two entities on interacting layers overlap this frame.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub a: Entity,
    pub a_layer: Layer,
    pub b: Entity,
    pub b_layer: Layer,
}

/// One entity's boxes this frame.
#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub entity: Entity,
    pub layer: Layer,
    pub aabb: Aabb,
    /// The box used against terrain (the same as `aabb` unless it has a
    /// [`TerrainHitbox`]).
    pub terrain_aabb: Aabb,
}

impl Collider {
    pub fn new(entity: Entity, layer: Layer, aabb: Aabb) -> Self {
        Self {
            entity,
            layer,
            aabb,
            terrain_aabb: aabb,
        }
    }
}

/// Every contact between the given boxes, pair by pair in [`PAIRS`] order.
pub fn find_contacts(colliders: &[Collider]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    for (a_layer, b_layer) in PAIRS {
        for a in colliders.iter().filter(|c| c.layer == a_layer) {
            for b in colliders.iter().filter(|c| c.layer == b_layer) {
                let b_box = if a_layer == Layer::Terrain {
                    &b.terrain_aabb
                } else {
                    &b.aabb
                };
                if a.aabb.overlaps(b_box) {
                    contacts.push(Contact {
                        a: a.entity,
                        a_layer,
                        b: b.entity,
                        b_layer,
                    });
                }
            }
        }
    }
    contacts
}

pub(super) fn detect(
    hitboxes: Query<(Entity, &Layer, &Hitbox, Option<&TerrainHitbox>, &Transform)>,
    mut contacts: MessageWriter<Contact>,
) {
    let colliders: Vec<_> = hitboxes
        .iter()
        .map(|(entity, layer, hitbox, terrain, t)| {
            let at = t.translation.truncate();
            let aabb = hitbox.aabb(at);
            Collider {
                entity,
                layer: *layer,
                aabb,
                terrain_aabb: terrain.map_or(aabb, |t| t.0.aabb(at)),
            }
        })
        .collect();
    contacts.write_batch(find_contacts(&colliders));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, y: f32, w: f32, h: f32) -> Aabb {
        Hitbox::new(w, h).aabb(Vec2::new(x, y))
    }

    #[test]
    fn overlapping_boxes_collide() {
        assert!(at(0.0, 0.0, 4.0, 4.0).overlaps(&at(3.0, 3.0, 4.0, 4.0)));
        assert!(at(0.0, 0.0, 10.0, 10.0).overlaps(&at(0.0, 0.0, 1.0, 1.0)));
    }

    #[test]
    fn separate_or_touching_boxes_do_not() {
        assert!(!at(0.0, 0.0, 4.0, 4.0).overlaps(&at(10.0, 0.0, 4.0, 4.0)));
        assert!(!at(0.0, 0.0, 4.0, 4.0).overlaps(&at(4.0, 0.0, 4.0, 4.0)));
        assert!(!at(0.0, 0.0, 4.0, 4.0).overlaps(&at(0.0, -4.0, 4.0, 4.0)));
    }

    #[test]
    fn hitbox_offset_moves_the_box_not_the_entity() {
        let hb = Hitbox::new(2.0, 2.0).with_offset(5.0, 0.0);
        let aabb = hb.aabb(Vec2::ZERO);
        assert_eq!(aabb.min, Vec2::new(4.0, -1.0));
        assert_eq!(aabb.max, Vec2::new(6.0, 1.0));
    }

    #[test]
    fn layer_table() {
        assert!(interacts(Layer::HeroShot, Layer::Enemy));
        assert!(interacts(Layer::Hero, Layer::EnemyShot));
        assert!(interacts(Layer::Hero, Layer::Pickup));
        assert!(!interacts(Layer::HeroShot, Layer::Hero));
        assert!(!interacts(Layer::Enemy, Layer::Enemy));
        assert!(!interacts(Layer::EnemyShot, Layer::Enemy));
        assert!(!interacts(Layer::Pickup, Layer::HeroShot));
        assert!(interacts(Layer::ShotBlocker, Layer::EnemyShot));
        assert!(!interacts(Layer::ShotBlocker, Layer::Enemy));
    }

    #[test]
    fn terrain_uses_the_terrain_hitbox_and_everything_else_the_small_one() {
        let hero = Entity::from_raw_u32(1).unwrap();
        let wall = Entity::from_raw_u32(2).unwrap();
        let bullet = Entity::from_raw_u32(3).unwrap();
        let mut ship = Collider::new(hero, Layer::Hero, at(0.0, 0.0, 4.0, 4.0));
        ship.terrain_aabb = at(0.0, 0.0, 12.0, 6.0);
        // The wall reaches the wide box; the bullet sits just above the small
        // box but inside the wide one.
        let boxes = [
            ship,
            Collider::new(wall, Layer::Terrain, at(8.0, 0.0, 6.0, 6.0)),
            Collider::new(bullet, Layer::EnemyShot, at(0.0, 3.0, 2.0, 2.0)),
        ];
        let contacts = find_contacts(&boxes);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].a, wall);
    }

    #[test]
    fn contacts_only_between_interacting_layers_in_pair_order() {
        let hero = Entity::from_raw_u32(1).unwrap();
        let enemy = Entity::from_raw_u32(2).unwrap();
        let shot = Entity::from_raw_u32(3).unwrap();
        let friend = Entity::from_raw_u32(4).unwrap();
        let boxes = [
            Collider::new(hero, Layer::Hero, at(0.0, 0.0, 8.0, 8.0)),
            Collider::new(enemy, Layer::Enemy, at(2.0, 0.0, 8.0, 8.0)),
            Collider::new(shot, Layer::HeroShot, at(1.0, 0.0, 2.0, 2.0)),
            Collider::new(friend, Layer::Enemy, at(100.0, 0.0, 8.0, 8.0)),
        ];
        let contacts = find_contacts(&boxes);
        assert_eq!(
            contacts,
            [
                Contact {
                    a: shot,
                    a_layer: Layer::HeroShot,
                    b: enemy,
                    b_layer: Layer::Enemy
                },
                Contact {
                    a: enemy,
                    a_layer: Layer::Enemy,
                    b: hero,
                    b_layer: Layer::Hero
                },
            ]
        );
    }
}
