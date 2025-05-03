use frame_support::pallet_prelude::*;
use frame_support::BoundedVec;
use scale_info::TypeInfo;

pub type CountryCode = BoundedVec<u8, ConstU32<5>>;
pub type Website = BoundedVec<u8, ConstU32<99>>;
pub type ProviderName = BoundedVec<u8, ConstU32<99>>;
pub type ResourceId = u32;
pub type StorageExtension = u64;
pub type VCPUExtension = u8;
pub const BASE_PRICE: u128 = 10_000_000_000; // for 1 week
pub const BASE_MEMORY_GB: u32 = 2;
pub const BASE_STORAGE_GB: u64 = 10;
pub const BASE_VCPU: u8 = 2;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub enum BillingPeriod {
    Weekly,
    Monthly,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub struct Resource<AccountId> {
    pub provider: AccountId,
    pub category: ResourceCategory,
    pub location: Region,
    pub is_available: bool,
    pub uptime_guarantee: u8, // Percentage of guaranteed uptime
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, DecodeWithMemTracking)]
pub struct CustomResourceSpecs {
    pub vcpu: u8,
    pub memory_gb: u32,
    pub storage_gb: u64,
    pub gpu_specs: Option<GPUSpecs>,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, DecodeWithMemTracking)]
pub struct GPUSpecs {
    pub gpu_count: u32,                          // Number of GPUs
    pub gpu_memory_gb: Option<u32>,              // Memory per GPU in Gigabytes (optional)
    pub gpu_model: BoundedVec<u8, ConstU32<99>>, // GPU model name
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub enum ResourceCategory {
    Nano(VCPUExtension, StorageExtension), // 2GB RAM, 10 GB Storage at least
    Micro(VCPUExtension, StorageExtension), // 4GB RAM, 20GB Storage
    Small(VCPUExtension, StorageExtension), // 8GB RAM, 8GB Storage
    Medium(VCPUExtension, StorageExtension), // 16GB RAM, 160GB Storage
    Large(VCPUExtension, StorageExtension), // 32GB RAM, 320GB Storage
    Custom(CustomResourceSpecs),           // Custom specs
}

impl ResourceCategory {
    pub fn price(&self, billing_period: &BillingPeriod) -> u128 {
        // TODO: Complete the pricing model (as of now will be onchain based (storage,constant)
        // without using runtime constant
        match billing_period {
            BillingPeriod::Weekly => BASE_PRICE,
            BillingPeriod::Monthly => BASE_PRICE * 4,
        }
    }

    pub fn specs(&self) -> (u8, u32, u64) {
        match self {
            ResourceCategory::Nano(vcpu_ext, storage_ext) => (
                BASE_VCPU + vcpu_ext,
                BASE_MEMORY_GB,
                BASE_STORAGE_GB + storage_ext,
            ),
            ResourceCategory::Micro(vcpu_ext, storage_ext) => (
                BASE_VCPU + vcpu_ext,
                BASE_MEMORY_GB * 2,
                BASE_STORAGE_GB * 2 + storage_ext,
            ),
            ResourceCategory::Small(vcpu_ext, storage_ext) => (
                (BASE_VCPU * 2) + vcpu_ext,
                BASE_MEMORY_GB * 4,
                BASE_STORAGE_GB * 4 + storage_ext,
            ),
            ResourceCategory::Medium(vcpu_ext, storage_ext) => (
                (BASE_VCPU * 2) + vcpu_ext,
                BASE_MEMORY_GB * 8,
                BASE_STORAGE_GB * 8 + storage_ext,
            ),
            ResourceCategory::Large(vcpu_ext, storage_ext) => (
                (BASE_VCPU * 4) + vcpu_ext,
                BASE_MEMORY_GB * 16,
                BASE_STORAGE_GB * 16 + storage_ext,
            ),
            ResourceCategory::Custom(specs) => (specs.vcpu, specs.memory_gb, specs.storage_gb),
        }
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub struct Rental<AccountId> {
    pub resource_id: ResourceId,
    pub renter: AccountId,
    pub start_block: u32,
    pub billing_period: BillingPeriod,
    pub last_paid_block: u32,
    pub is_active: bool,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub struct Region {
    pub country_code: CountryCode,
    pub city: BoundedVec<u8, ConstU32<32>>,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
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

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub enum ProviderStatus {
    Active,
    Inactive,
    Suspended,
    Terminated,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub struct BootstrapperInfo<AccountId> {
    pub bootstrapper_info: AccountId,
    pub bootstrap_type: BootstrapperType,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Copy, DecodeWithMemTracking)]
pub enum BootstrapperType {
    Kollectyve,
    Else,
}
