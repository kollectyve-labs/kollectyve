use frame_support::pallet_prelude::*;
use frame_support::BoundedVec;
use scale_info::TypeInfo;

pub type ResourceId = u32;
pub type CountryCode = BoundedVec<u8, ConstU32<5>>;
//pub type Location = BoundedVec<u8, ConstU32<99>>;
//pub type Locations = BoundedVec<Location, ConstU32<7>>;
pub type Website = BoundedVec<u8, ConstU32<99>>;
pub type ProviderName = BoundedVec<u8, ConstU32<99>>;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BillingPeriod {
    Weekly,
    Monthly,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ResourceCategory {
    Basic,    // 1 CPU, 1GB RAM, 25GB Storage
    Standard, // 2 CPU, 2GB RAM, 50GB Storage
    Enhanced, // 4 CPU, 8GB RAM, 160GB Storage
    Premium,  // 8 CPU, 16GB RAM, 320GB Storage
}

impl ResourceCategory {
    // Get the specifications for this category
    pub fn specs(&self) -> (u32, u32, u32) {
        match self {
            ResourceCategory::Basic => (1, 1, 25),
            ResourceCategory::Standard => (2, 2, 50),
            ResourceCategory::Enhanced => (4, 8, 160),
            ResourceCategory::Premium => (8, 16, 320),
        }
    }

    // Fixed monthly prices
    pub fn price(&self, period: &BillingPeriod) -> u128 {
        let monthly_price = match self {
            ResourceCategory::Basic => 5_000_000_000,     // 5 tokens
            ResourceCategory::Standard => 10_000_000_000, // 10 tokens
            ResourceCategory::Enhanced => 20_000_000_000, // 20 tokens
            ResourceCategory::Premium => 40_000_000_000,  // 40 tokens
        };

        match period {
            BillingPeriod::Monthly => monthly_price,
            BillingPeriod::Weekly => monthly_price / 4, // Divide by 4 for weekly price
        }
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Resource<AccountId> {
    pub provider: AccountId,
    pub category: ResourceCategory,
    pub location: ResourceLocation,
    pub is_available: bool,
    pub provider_tier: ProviderTier,
    pub uptime_guarantee: u8, // Percentage of guaranteed uptime
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ResourceLocation {
    pub region: Region,
    // TODO: To update when mesures in place
    // pub network_quality: NetworkQuality,
}

/// Average Latency (avg_latency_ms):  represents the time it takes for data to travel from one point to another in the network.
/// Packet Loss Percent (packet_loss_percent): refers to the percentage of packets that fail to reach their destination.
/// Jitter (jitter_ms): measures the variation in delay times for data packets being transmitted over a network.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct NetworkQuality {
    pub avg_latency_ms: u32,
    pub packet_loss_percent: u8,
    pub jitter_ms: u32,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Rental<AccountId> {
    pub resource_id: ResourceId,
    pub renter: AccountId,
    pub start_block: u32,
    pub billing_period: BillingPeriod,
    pub last_paid_block: u32,
    pub is_active: bool,
}

#[derive(Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ProviderInfo<AccountId> {
    pub account: AccountId,
    pub name: ProviderName,       // Provider's name/organization
    pub website: Option<Website>, // Optional website URL
    pub total_resources: u32,     // Total resources across all categories
    pub reputation_score: u32,    // Score from 0-100
    pub registration_block: u32,  // Block number when registered
    pub last_updated: u32,        // Last update block number
    pub status: ProviderStatus,   // Current status
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProviderStatus {
    Active,
    Inactive,
    Suspended,
    Terminated,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProviderTier {
    Mobile {
        device_type: DeviceType,
        is_cluster: bool,
        cluster_size: Option<u16>,
    },
    Personal {
        computer_type: ComputerType,
        is_cluster: bool,
        cluster_size: Option<u16>,
    },
    DataCenter {
        tier_level: DataCenterTier,
        rack_count: u16,
    },
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum DeviceType {
    Smartphone,
    Tablet,
    EdgeDevice,
    Custom(BoundedVec<u8, ConstU32<32>>), // For future device types
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ComputerType {
    Desktop,
    Workstation,
    Server,
    Custom(BoundedVec<u8, ConstU32<32>>), // For future computer types
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum DataCenterTier {
    Tier1, // Basic
    Tier2, // Redundant Components
    Tier3, // Concurrently Maintainable
    Tier4, // Fault Tolerant
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Region {
    pub country_code: CountryCode,
    pub city: BoundedVec<u8, ConstU32<32>>,
    pub coordinates: Option<Coordinates>,
    pub connection_type: ConnectionType,
    pub power_backup: bool,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Coordinates {
    pub latitude: i32,  // Multiplied by 1_000_000 to avoid floats
    pub longitude: i32, // Multiplied by 1_000_000 to avoid floats
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ConnectionType {
    Fiber { speed_mbps: u32 },
    Broadband { speed_mbps: u32 },
    Mobile4G { speed_mbps: u32 },
    Mobile5G { speed_mbps: u32 },
    Satellite { latency_ms: u32 },
    Other { speed_mbps: u32 },
}
