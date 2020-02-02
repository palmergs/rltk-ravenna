extern crate rltk;
use rltk::{ RGB };

extern crate specs;
use specs::prelude::*;
use specs::saveload::{ Marker, ConvertSaveload };
use specs::error::NoError;

extern crate specs_derive;

extern crate serde;
use serde::{ Serialize, Deserialize };


#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Player {}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32
}


#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Monster {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct SufferDamage {
    pub amount: i32
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Item {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MagicMapper {}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Shield }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct ProvidesHealing {
    pub heal_amount: i32
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Viewshed {
    pub tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BlocksTile {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}
