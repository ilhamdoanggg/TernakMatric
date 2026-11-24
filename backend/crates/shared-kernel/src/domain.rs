use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpeciesCategory {
    Fish,
    Crustacean,
    Mollusk,
    Poultry,
    Mammal,
    Reptile,
    Insect,
    Other,
}

impl SpeciesCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fish => "fish",
            Self::Crustacean => "crustacean",
            Self::Mollusk => "mollusk",
            Self::Poultry => "poultry",
            Self::Mammal => "mammal",
            Self::Reptile => "reptile",
            Self::Insect => "insect",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnitHabitat {
    Water,
    Land,
    Cage,
    Pasture,
    Hive,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapacityUnit {
    Head,
    Kg,
    M3,
    M2,
    Tray,
    Crate,
    Colony,
    Hive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Planned,
    Active,
    Harvesting,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    InitialStock,
    AdditionalStock,
    Birth,
    Purchase,
    Mortality,
    Cull,
    Harvest,
    EggCollection,
    MilkCollection,
    TransferIn,
    TransferOut,
    Sale,
    Adjustment,
    Reversal,
}

impl MovementType {
    /// Returns whether this movement modifies live headcount in `population_balances`.
    pub fn affects_live_headcount(&self) -> bool {
        !matches!(self, Self::EggCollection | Self::MilkCollection)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InitialStock => "initial_stock",
            Self::AdditionalStock => "additional_stock",
            Self::Birth => "birth",
            Self::Purchase => "purchase",
            Self::Mortality => "mortality",
            Self::Cull => "cull",
            Self::Harvest => "harvest",
            Self::EggCollection => "egg_collection",
            Self::MilkCollection => "milk_collection",
            Self::TransferIn => "transfer_in",
            Self::TransferOut => "transfer_out",
            Self::Sale => "sale",
            Self::Adjustment => "adjustment",
            Self::Reversal => "reversal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Meat,
    Egg,
    Milk,
    Fry,
    Cull,
    Offspring,
    Other,
}

impl OutputType {
    pub fn reduces_population_when_harvested(&self) -> bool {
        matches!(self, Self::Meat | Self::Fry | Self::Cull | Self::Offspring)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentCategory {
    Water,
    Air,
    Soil,
    Housing,
    Feed,
    Other,
}
