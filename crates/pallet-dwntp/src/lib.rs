#![cfg_attr(not(feature = "std"), no_std)]

//! # DWNTP Pallet
//!
//! A Polkadot-SDK pallet for logging RTU control events in the DWNTP smart grid network.
//! This pallet stores immutable control events submitted by authorized Master Terminal Units (MTUs).

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::traits::UnixTime;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the maximum length of an RTU ID.
        #[pallet::constant]
        type MaxRtuIdLen: Get<u32>;

        /// Type representing the maximum length of an event name.
        #[pallet::constant]
        type MaxEventNameLen: Get<u32>;

        /// Type representing the maximum length of an event description.
        #[pallet::constant]
        type MaxEventDescLen: Get<u32>;

        /// Time provider for recording on-chain timestamps.
        type TimeProvider: UnixTime;
    }

    /// An RTU control event stored on-chain.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct OnChainControlEvent<T: Config> {
        /// The MTU that submitted this event.
        pub source_mtu: T::AccountId,
        /// Identifier of the target RTU.
        pub rtu_id: BoundedVec<u8, T::MaxRtuIdLen>,
        /// Name/type of the control event.
        pub event_name: BoundedVec<u8, T::MaxEventNameLen>,
        /// Description of the event and its parameters.
        pub event_description: BoundedVec<u8, T::MaxEventDescLen>,
        /// Timestamp when the event was created/submitted (Unix time in milliseconds).
        pub event_timestamp: u64,
        /// Timestamp when the event was recorded on-chain (Unix time in milliseconds).
        pub on_chain_timestamp: u64,
    }

    /// Storage map for control events.
    /// Maps a 32-byte hash (event ID) to the control event details.
    #[pallet::storage]
    #[pallet::getter(fn control_events)]
    pub type ControlEvents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Event ID (e.g., SHA-256 hash)
        OnChainControlEvent<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new RTU control event was successfully logged on-chain.
        /// [event_id, source_mtu]
        ControlEventLogged {
            event_id: [u8; 32],
            source_mtu: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The event ID has already been logged (duplicate event).
        EventAlreadyExists,
        /// The RTU ID provided is invalid or exceeds maximum length.
        InvalidRtuId,
        /// The event name provided is invalid or exceeds maximum length.
        InvalidEventName,
        /// The event description provided is invalid or exceeds maximum length.
        InvalidEventDescription,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new RTU control event to the blockchain.
        ///
        /// The event ID must be a unique 32-byte hash (e.g., SHA-256 of the event contents).
        /// All string fields must be provided as byte vectors and fit within configured bounds.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::default())] // TODO: Replace with proper benchmarking weights
        pub fn log_control_event(
            origin: OriginFor<T>,
            event_id: [u8; 32],
            rtu_id: Vec<u8>,
            event_name: Vec<u8>,
            event_description: Vec<u8>,
            event_timestamp: u64,
        ) -> DispatchResult {
            // Ensure the caller is signed (it must be an authorized MTU)
            // Note: In Phase 4, we will add verification to ensure `source_mtu` is an authorized validator
            let source_mtu = ensure_signed(origin)?;

            // Ensure the event hasn't already been logged
            ensure!(
                !ControlEvents::<T>::contains_key(event_id),
                Error::<T>::EventAlreadyExists
            );

            // Bound the incoming vectors
            let bounded_rtu_id: BoundedVec<u8, T::MaxRtuIdLen> =
                rtu_id.try_into().map_err(|_| Error::<T>::InvalidRtuId)?;

            let bounded_event_name: BoundedVec<u8, T::MaxEventNameLen> = event_name
                .try_into()
                .map_err(|_| Error::<T>::InvalidEventName)?;

            let bounded_event_desc: BoundedVec<u8, T::MaxEventDescLen> = event_description
                .try_into()
                .map_err(|_| Error::<T>::InvalidEventDescription)?;

            // Get current on-chain timestamp (milliseconds since epoch)
            let on_chain_timestamp = T::TimeProvider::now().as_millis() as u64;

            // Construct the on-chain event
            let control_event = OnChainControlEvent {
                source_mtu: source_mtu.clone(),
                rtu_id: bounded_rtu_id,
                event_name: bounded_event_name,
                event_description: bounded_event_desc,
                event_timestamp,
                on_chain_timestamp,
            };

            // Store the event immutably
            ControlEvents::<T>::insert(event_id, control_event);

            // Emit the success event
            Self::deposit_event(Event::ControlEventLogged {
                event_id,
                source_mtu,
            });

            Ok(())
        }
    }
}
