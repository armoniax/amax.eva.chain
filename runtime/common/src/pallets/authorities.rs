use frame_support::ensure;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MaxAuthorities: Get<u32>;
        // type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Prepare a new group authorities, and it will take effort after at a new session.
        PrepareNewAuthorities(Vec<T::AccountId>),
        ChangeAuthorities {
            index: sp_staking::SessionIndex,
            authorities: Vec<T::AccountId>,
        },
        ForceSwitchGroup,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The new authority group is same as current one, we do nothing for this.
        SameAuthorityGroup,
    }

    #[pallet::storage]
    #[pallet::getter(fn authorities)]
    pub type Authorities<T: Config> = StorageValue<_, Vec<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn switch_new_group)]
    pub type SwitchNewGroup<T> = StorageValue<_, bool>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set new authorities group, and this group will be commit to the consensus system in next
        /// session.
        #[pallet::weight(
            (42_131_000 as Weight)
                .saturating_add(T::DbWeight::get().reads(1 as Weight))
                .saturating_add(T::DbWeight::get().writes(1 as Weight))
        )]
        pub fn new_authorities(
            origin: OriginFor<T>,
            authorities: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let mut authorities = authorities;
            authorities.sort();

            Authorities::<T>::try_mutate::<_, Error<T>, _>(|current| {
                if let Some(old) = current {
                    ensure!(*old != authorities, Error::<T>::SameAuthorityGroup);
                    // set to new group;
                    *old = authorities.clone();
                }
                Ok(())
            })?;
            SwitchNewGroup::<T>::put(true);
            Self::deposit_event(Event::<T>::PrepareNewAuthorities(authorities));
            Ok(())
        }
        /// Force to switch authorities group for current authorities storage in next session,
        /// even there is no change in current group.
        #[pallet::weight(42_131_000 as Weight)]
        pub fn force_switch(origin: OriginFor<T>) -> DispatchResult {
            ensure_none(origin)?;
            // set flag for switch.
            SwitchNewGroup::<T>::put(true);
            Self::deposit_event(Event::<T>::ForceSwitchGroup);
            Ok(())
        }
    }
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
    fn new_session(index: sp_staking::SessionIndex) -> Option<Vec<T::AccountId>> {
        if !Self::switch_new_group().unwrap_or(false) {
            return None
        }

        let authorities = Self::authorities();
        if let Some(ref auths) = authorities {
            Self::deposit_event(Event::<T>::ChangeAuthorities { index, authorities: auths.clone() })
        }
        // clear flag for switch.
        SwitchNewGroup::<T>::take();
        authorities
    }

    fn end_session(_: sp_staking::SessionIndex) {}

    fn start_session(_: sp_staking::SessionIndex) {}
}

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for Pallet<T> {
    fn convert(a: T::AccountId) -> Option<T::AccountId> {
        Some(a)
    }
}
