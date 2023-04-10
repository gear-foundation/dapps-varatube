#![no_std]

use gmeta::{In, Metadata};
use gstd::{prelude::*, ActorId};

pub type TokenData = (ActorId, Price);
pub type Price = u128;

/// Subscription contract metadata
pub struct SubscriptionMetadata;

impl Metadata for SubscriptionMetadata {
    type Init = In<TokenData>;
    type Handle = In<Actions>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = SubscriptionState;
}

/// Actions callable by a user on the subscription contract
#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Actions {
    /// Create a new subscription for a user `ActorId` for a `Period` of time.
    /// Automatically renewed if `with_renewal` is enabled.
    RegisterSubscription {
        currency_id: ActorId,
        period: Period,
        with_renewal: bool,
    },
    /// Update (renew or end) an existing subscription.
    UpdateSubscription { subscriber: ActorId },
    /// Cancel existing subscription
    CancelSubscription,
    /// Initialize or delete a pending subscription (which can be the case
    /// if `RegisterSubscription` action failed due to out-of-gas)
    ManagePendingSubscription { enable: bool },
}

/// Set of time periods for which a subscription can be purchased
/// in context of the sucbscription contract.
#[derive(Debug, Clone, Copy, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Period {
    Year,
    NineMonths,
    SixMonths,
    ThreeMonths,
    #[default]
    Month,
}

impl Period {
    // TODO [cleanness] Must be changeable
    const TARGET_BLOCK_TIME: u32 = Self::SECOND;
    const SECOND: u32 = 1;

    pub fn minimal_unit() -> Self {
        Self::Month
    }

    pub fn to_units(&self) -> u128 {
        match self {
            Period::Year => 12,
            Period::NineMonths => 9,
            Period::SixMonths => 6,
            Period::ThreeMonths => 3,
            Period::Month => 1,
        }
    }

    pub fn to_blocks(&self) -> u32 {
        self.to_secs() / Self::TARGET_BLOCK_TIME
    }

    pub fn to_millis(&self) -> u64 {
        self.to_secs() as u64 * 1000
    }

    fn to_secs(&self) -> u32 {
        match self {
            Period::Year => Self::Month.to_secs() * 12,
            Period::NineMonths => Self::Month.to_secs() * 9,
            Period::SixMonths => Self::Month.to_secs() * 6,
            Period::ThreeMonths => Self::Month.to_secs() * 3,
            Period::Month => Self::SECOND * 30,
        }
    }
}

/// State of the subscription contract
#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriptionState {
    pub subscribers: BTreeMap<ActorId, SubscriberData>,
    pub currencies: BTreeMap<ActorId, Price>,
}

type V = (BTreeMap<ActorId, SubscriberData>, BTreeMap<ActorId, Price>);

impl From<V> for SubscriptionState {
    fn from(value: V) -> Self {
        let (subscribers, currencies) = value;
        SubscriptionState {
            subscribers,
            currencies,
        }
    }
}

/// Subscriber's data
#[derive(Debug, Clone, Copy, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberData {
    /// Id of the payment method.
    pub currency_id: ActorId,
    /// Subscription period
    pub period: Period,
    // TODO [optimization] this must be calculated off-chain
    /// Subscription start timestamp and block number.
    ///
    /// If `None`, means that subscriber has paid for the
    /// subscription, but didn't succeed sending delayed
    /// message for subscription check/renewal.
    pub subscription_start: Option<(u64, u32)>,
    // TODO [optimization] this must be calculated off-chain
    /// Subscription renewal date.
    ///
    /// If None, then no renewal desired.
    pub renewal_date: Option<(u64, u32)>,
}

/// Subscriber's state
#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberDataState {
    pub is_active: bool,
    pub start_date: u64,
    pub start_block: u32,
    pub end_date: u64,
    pub end_block: u32,
    pub period: Period,
    pub will_renew: bool,
    pub price: u128,
}
