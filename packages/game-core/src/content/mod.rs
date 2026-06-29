use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeedId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusId(pub u8);

#[derive(Clone, Debug)]
pub struct TerrainDef {
    pub id: String,
    pub walkable: bool,
    pub color: u32,
}

#[derive(Clone, Debug)]
pub struct NeedDef {
    pub id: String,
    pub label: String,
    pub max: f32,
    pub decay_per_sec: f32,
    pub critical_threshold: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyCondition {
    BelowThreshold,
}

#[derive(Clone, Debug)]
pub struct ApplyWhen {
    pub need: NeedId,
    pub condition: ApplyCondition,
}

#[derive(Clone, Debug)]
pub enum StatusEffect {
    TaskPriority { task: TaskKindRef, priority: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKindRef {
    Eat,
    Sleep,
    Build,
}

#[derive(Clone, Debug)]
pub struct StatusDef {
    pub id: String,
    pub label: String,
    pub apply_when: ApplyWhen,
    pub effects: Vec<StatusEffect>,
}

#[derive(Clone, Debug)]
pub enum SpawnPrimitive {
    Supply { resource: String, amount: u8 },
    Reservation { kind: ReservationKind },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationKind {
    SingleOccupant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Adjacent,
    OnTile,
}

#[derive(Clone, Debug)]
pub enum InteractionEffect {
    RestoreNeed { need: NeedId, amount: f32 },
    ConsumeSupply { resource: String, amount: u8 },
}

#[derive(Clone, Debug)]
pub struct InteractionDef {
    pub mode: InteractionMode,
    pub duration_sec: f32,
    pub effects: Vec<InteractionEffect>,
}

#[derive(Clone, Debug)]
pub struct BuildingDef {
    pub id: String,
    pub label: String,
    pub work_required: f32,
    pub work_to_deconstruct: f32,
    pub blocks_movement: bool,
    pub blocks_settle: bool,
    pub buildable: bool,
    pub color: u32,
    pub on_complete: Vec<SpawnPrimitive>,
    pub interactions: Vec<InteractionDef>,
}

#[derive(Resource, Clone, Debug)]
pub struct ContentRegistry {
    pub terrain: Vec<TerrainDef>,
    pub buildings: Vec<BuildingDef>,
    pub needs: Vec<NeedDef>,
    pub statuses: Vec<StatusDef>,
    terrain_by_str: HashMap<String, TerrainId>,
    building_by_str: HashMap<String, BuildingId>,
    need_by_str: HashMap<String, NeedId>,
    status_by_str: HashMap<String, StatusId>,
    pub food_need: NeedId,
    pub sleep_need: NeedId,
    pub hungry_status: StatusId,
    pub wants_sleep_status: StatusId,
    pub grass_terrain: TerrainId,
    pub water_terrain: TerrainId,
    pub sand_terrain: TerrainId,
    pub berry_bush_building: BuildingId,
    pub bed_building: BuildingId,
    pub wall_building: BuildingId,
}

#[derive(Debug, Deserialize)]
struct ContentPackRaw {
    needs: Vec<NeedDefRaw>,
    statuses: Vec<StatusDefRaw>,
    buildings: Vec<BuildingDefRaw>,
    terrain: Vec<TerrainDefRaw>,
}

#[derive(Debug, Deserialize)]
struct NeedDefRaw {
    id: String,
    label: String,
    max: f32,
    decay_per_sec: f32,
    critical_threshold: f32,
}

#[derive(Debug, Deserialize)]
struct ApplyWhenRaw {
    need: String,
    condition: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StatusEffectRaw {
    TaskPriority { task: String, priority: i32 },
}

#[derive(Debug, Deserialize)]
struct StatusDefRaw {
    id: String,
    label: String,
    apply_when: ApplyWhenRaw,
    effects: Vec<StatusEffectRaw>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SpawnPrimitiveRaw {
    Supply { resource: String, amount: u8 },
    Reservation { kind: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InteractionEffectRaw {
    RestoreNeed { need: String, amount: f32 },
    ConsumeSupply { resource: String, amount: u8 },
}

#[derive(Debug, Deserialize)]
struct InteractionDefRaw {
    mode: String,
    duration_sec: f32,
    effects: Vec<InteractionEffectRaw>,
}

#[derive(Debug, Deserialize)]
struct BuildingDefRaw {
    id: String,
    label: String,
    work_required: f32,
    #[serde(default)]
    work_to_deconstruct: Option<f32>,
    blocks_movement: bool,
    blocks_settle: bool,
    #[serde(default = "default_true")]
    buildable: bool,
    color: u32,
    #[serde(default)]
    on_complete: Vec<SpawnPrimitiveRaw>,
    #[serde(default)]
    interactions: Vec<InteractionDefRaw>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct TerrainDefRaw {
    id: String,
    walkable: bool,
    color: u32,
}

#[derive(Debug)]
pub struct ContentError(String);

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ContentRegistry {
    pub fn from_json(json: &str) -> Result<Self, ContentError> {
        let raw: ContentPackRaw =
            serde_json::from_str(json).map_err(|e| ContentError(format!("invalid JSON: {e}")))?;
        Self::from_pack(raw)
    }

    fn from_pack(raw: ContentPackRaw) -> Result<Self, ContentError> {
        let mut need_by_str = HashMap::new();
        let mut needs = Vec::new();
        for (idx, n) in raw.needs.into_iter().enumerate() {
            if need_by_str.contains_key(&n.id) {
                return Err(ContentError(format!("duplicate need id: {}", n.id)));
            }
            if n.max <= 0.0 {
                return Err(ContentError(format!("need {} max must be positive", n.id)));
            }
            need_by_str.insert(n.id.clone(), NeedId(idx as u8));
            needs.push(NeedDef {
                id: n.id,
                label: n.label,
                max: n.max,
                decay_per_sec: n.decay_per_sec,
                critical_threshold: n.critical_threshold,
            });
        }

        let mut terrain_by_str = HashMap::new();
        let mut terrain = Vec::new();
        for (idx, t) in raw.terrain.into_iter().enumerate() {
            if terrain_by_str.contains_key(&t.id) {
                return Err(ContentError(format!("duplicate terrain id: {}", t.id)));
            }
            terrain_by_str.insert(t.id.clone(), TerrainId(idx as u16));
            terrain.push(TerrainDef {
                id: t.id,
                walkable: t.walkable,
                color: t.color,
            });
        }

        let mut building_by_str = HashMap::new();
        let mut buildings = Vec::new();
        for (idx, b) in raw.buildings.into_iter().enumerate() {
            if building_by_str.contains_key(&b.id) {
                return Err(ContentError(format!("duplicate building id: {}", b.id)));
            }
            let on_complete = b
                .on_complete
                .into_iter()
                .map(|p| parse_spawn_primitive(p))
                .collect::<Result<Vec<_>, _>>()?;
            let interactions = b
                .interactions
                .into_iter()
                .map(|i| parse_interaction(&i, &need_by_str))
                .collect::<Result<Vec<_>, _>>()?;
            building_by_str.insert(b.id.clone(), BuildingId(idx as u16));
            buildings.push(BuildingDef {
                id: b.id,
                label: b.label,
                work_required: b.work_required,
                work_to_deconstruct: b.work_to_deconstruct.unwrap_or(b.work_required),
                blocks_movement: b.blocks_movement,
                blocks_settle: b.blocks_settle,
                buildable: b.buildable,
                color: b.color,
                on_complete,
                interactions,
            });
        }

        let mut status_by_str = HashMap::new();
        let mut statuses = Vec::new();
        for (idx, s) in raw.statuses.into_iter().enumerate() {
            if status_by_str.contains_key(&s.id) {
                return Err(ContentError(format!("duplicate status id: {}", s.id)));
            }
            let need = need_by_str
                .get(&s.apply_when.need)
                .copied()
                .ok_or_else(|| {
                    ContentError(format!(
                        "status {} references unknown need {}",
                        s.id, s.apply_when.need
                    ))
                })?;
            let condition = match s.apply_when.condition.as_str() {
                "below_threshold" => ApplyCondition::BelowThreshold,
                other => {
                    return Err(ContentError(format!(
                        "status {} unknown condition: {other}",
                        s.id
                    )))
                }
            };
            let effects = s
                .effects
                .into_iter()
                .map(|e| parse_status_effect(e))
                .collect::<Result<Vec<_>, _>>()?;
            status_by_str.insert(s.id.clone(), StatusId(idx as u8));
            statuses.push(StatusDef {
                id: s.id,
                label: s.label,
                apply_when: ApplyWhen { need, condition },
                effects,
            });
        }

        let food_need = *need_by_str
            .get("food")
            .ok_or_else(|| ContentError("base pack missing need: food".into()))?;
        let sleep_need = *need_by_str
            .get("sleep")
            .ok_or_else(|| ContentError("base pack missing need: sleep".into()))?;
        let hungry_status = *status_by_str
            .get("hungry")
            .ok_or_else(|| ContentError("base pack missing status: hungry".into()))?;
        let wants_sleep_status = *status_by_str
            .get("wants_sleep")
            .ok_or_else(|| ContentError("base pack missing status: wants_sleep".into()))?;
        let grass_terrain = *terrain_by_str
            .get("grass")
            .ok_or_else(|| ContentError("base pack missing terrain: grass".into()))?;
        let water_terrain = *terrain_by_str
            .get("water")
            .ok_or_else(|| ContentError("base pack missing terrain: water".into()))?;
        let sand_terrain = *terrain_by_str
            .get("sand")
            .ok_or_else(|| ContentError("base pack missing terrain: sand".into()))?;
        let berry_bush_building = *building_by_str
            .get("berry_bush")
            .ok_or_else(|| ContentError("base pack missing building: berry_bush".into()))?;
        let bed_building = *building_by_str
            .get("bed")
            .ok_or_else(|| ContentError("base pack missing building: bed".into()))?;
        let wall_building = *building_by_str
            .get("wall")
            .ok_or_else(|| ContentError("base pack missing building: wall".into()))?;

        Ok(Self {
            terrain,
            buildings,
            needs,
            statuses,
            terrain_by_str,
            building_by_str,
            need_by_str,
            status_by_str,
            food_need,
            sleep_need,
            hungry_status,
            wants_sleep_status,
            grass_terrain,
            water_terrain,
            sand_terrain,
            berry_bush_building,
            bed_building,
            wall_building,
        })
    }

    pub fn terrain_id(&self, id: &str) -> Option<TerrainId> {
        self.terrain_by_str.get(id).copied()
    }

    pub fn building_id(&self, id: &str) -> Option<BuildingId> {
        self.building_by_str.get(id).copied()
    }

    pub fn need_id(&self, id: &str) -> Option<NeedId> {
        self.need_by_str.get(id).copied()
    }

    pub fn status_id(&self, id: &str) -> Option<StatusId> {
        self.status_by_str.get(id).copied()
    }

    pub fn terrain_str(&self, id: TerrainId) -> &str {
        &self.terrain[id.0 as usize].id
    }

    pub fn building_str(&self, id: BuildingId) -> &str {
        &self.buildings[id.0 as usize].id
    }

    pub fn need_str(&self, id: NeedId) -> &str {
        &self.needs[id.0 as usize].id
    }

    pub fn status_str(&self, id: StatusId) -> &str {
        &self.statuses[id.0 as usize].id
    }

    pub fn terrain_def(&self, id: TerrainId) -> &TerrainDef {
        &self.terrain[id.0 as usize]
    }

    pub fn building_def(&self, id: BuildingId) -> &BuildingDef {
        &self.buildings[id.0 as usize]
    }

    pub fn need_def(&self, id: NeedId) -> &NeedDef {
        &self.needs[id.0 as usize]
    }

    pub fn status_def(&self, id: StatusId) -> &StatusDef {
        &self.statuses[id.0 as usize]
    }

    pub fn work_required(&self, id: BuildingId) -> f32 {
        self.building_def(id).work_required
    }

    pub fn work_to_deconstruct(&self, id: BuildingId) -> f32 {
        self.building_def(id).work_to_deconstruct
    }

    pub fn blocks_movement(&self, id: BuildingId) -> bool {
        self.building_def(id).blocks_movement
    }

    pub fn blocks_settle(&self, id: BuildingId) -> bool {
        self.building_def(id).blocks_settle
    }

    pub fn has_supply_on_complete(&self, id: BuildingId) -> bool {
        self.building_def(id)
            .on_complete
            .iter()
            .any(|p| matches!(p, SpawnPrimitive::Supply { .. }))
    }

    pub fn supply_amount_on_complete(&self, id: BuildingId) -> Option<u8> {
        for p in &self.building_def(id).on_complete {
            if let SpawnPrimitive::Supply { amount, .. } = p {
                return Some(*amount);
            }
        }
        None
    }

    pub fn has_reservation_on_complete(&self, id: BuildingId) -> bool {
        self.building_def(id)
            .on_complete
            .iter()
            .any(|p| matches!(p, SpawnPrimitive::Reservation { .. }))
    }

    pub fn buildable_buildings(&self) -> impl Iterator<Item = (BuildingId, &BuildingDef)> {
        self.buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.buildable)
            .map(|(i, b)| (BuildingId(i as u16), b))
    }

    pub fn task_for_status(&self, status: StatusId) -> Option<TaskKindRef> {
        for effect in &self.status_def(status).effects {
            match effect {
                StatusEffect::TaskPriority { task, .. } => return Some(*task),
            }
        }
        None
    }

    pub fn sleep_interaction(&self, id: BuildingId) -> Option<&InteractionDef> {
        self.building_def(id)
            .interactions
            .iter()
            .find(|i| i.mode == InteractionMode::OnTile)
    }

    pub fn eat_interaction(&self, id: BuildingId) -> Option<&InteractionDef> {
        self.building_def(id)
            .interactions
            .iter()
            .find(|i| i.mode == InteractionMode::Adjacent)
    }
}

fn parse_spawn_primitive(raw: SpawnPrimitiveRaw) -> Result<SpawnPrimitive, ContentError> {
    match raw {
        SpawnPrimitiveRaw::Supply { resource, amount } => {
            Ok(SpawnPrimitive::Supply { resource, amount })
        }
        SpawnPrimitiveRaw::Reservation { kind } => {
            let kind = match kind.as_str() {
                "single_occupant" => ReservationKind::SingleOccupant,
                other => return Err(ContentError(format!("unknown reservation kind: {other}"))),
            };
            Ok(SpawnPrimitive::Reservation { kind })
        }
    }
}

fn parse_interaction(
    raw: &InteractionDefRaw,
    needs: &HashMap<String, NeedId>,
) -> Result<InteractionDef, ContentError> {
    let mode = match raw.mode.as_str() {
        "adjacent" => InteractionMode::Adjacent,
        "on_tile" => InteractionMode::OnTile,
        other => return Err(ContentError(format!("unknown interaction mode: {other}"))),
    };
    let effects = raw
        .effects
        .iter()
        .map(|e| parse_interaction_effect(e, needs))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(InteractionDef {
        mode,
        duration_sec: raw.duration_sec,
        effects,
    })
}

fn parse_interaction_effect(
    raw: &InteractionEffectRaw,
    needs: &HashMap<String, NeedId>,
) -> Result<InteractionEffect, ContentError> {
    match raw {
        InteractionEffectRaw::RestoreNeed { need, amount } => {
            let need_id = *needs
                .get(need)
                .ok_or_else(|| ContentError(format!("interaction references unknown need: {need}")))?;
            Ok(InteractionEffect::RestoreNeed {
                need: need_id,
                amount: *amount,
            })
        }
        InteractionEffectRaw::ConsumeSupply { resource, amount } => Ok(
            InteractionEffect::ConsumeSupply {
                resource: resource.clone(),
                amount: *amount,
            },
        ),
    }
}

fn parse_status_effect(raw: StatusEffectRaw) -> Result<StatusEffect, ContentError> {
    match raw {
        StatusEffectRaw::TaskPriority { task, priority } => {
            let task = match task.as_str() {
                "eat" => TaskKindRef::Eat,
                "sleep" => TaskKindRef::Sleep,
                "build" => TaskKindRef::Build,
                other => return Err(ContentError(format!("unknown task in status effect: {other}"))),
            };
            Ok(StatusEffect::TaskPriority { task, priority })
        }
    }
}

pub fn base_content_json() -> &'static str {
    include_str!("../../../../content/base/pack.json")
}

pub fn base_content() -> ContentRegistry {
    ContentRegistry::from_json(base_content_json()).expect("base content pack must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_base_pack() {
        let registry = base_content();
        assert_eq!(registry.needs.len(), 2);
        assert_eq!(registry.buildings.len(), 3);
        assert_eq!(registry.terrain.len(), 3);
        assert_eq!(registry.statuses.len(), 2);
    }

    #[test]
    fn o1_def_access() {
        let registry = base_content();
        let id = registry.berry_bush_building;
        assert_eq!(registry.building_def(id).work_required, 40.0);
        assert!(registry.blocks_settle(id));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let json = r#"{"needs":[],"statuses":[],"terrain":[],"buildings":[{"id":"x","label":"X","work_required":1,"blocks_movement":false,"blocks_settle":false,"color":0,"on_complete":[],"interactions":[]},{"id":"x","label":"Y","work_required":1,"blocks_movement":false,"blocks_settle":false,"color":0,"on_complete":[],"interactions":[]}]}"#;
        assert!(ContentRegistry::from_json(json).is_err());
    }

    #[test]
    fn rejects_unknown_need_ref() {
        let json = r#"{"needs":[],"statuses":[{"id":"s","label":"S","apply_when":{"need":"missing","condition":"below_threshold"},"effects":[]}],"terrain":[],"buildings":[]}"#;
        assert!(ContentRegistry::from_json(json).is_err());
    }
}
