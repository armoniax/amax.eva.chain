use frame_support::ensure;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

pub use self::pallet::*;

const TARGET: &str = "runtime::authorities";

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
    pub trait Config: frame_system::Config + pallet_session::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MaxAuthorities: Get<u32>;
        // type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Prepare a new group authorities, and it will take effort after at a new session.
        PrepareNewAuthorities { group: Vec<T::AccountId> },
        /// Change authorities at Session index.
        ChangeAuthorities { index: sp_staking::SessionIndex, authorities: Vec<T::AccountId> },
        /// Force switch group in next session.
        ForceSwitchGroup,
        /// Clear switch group flag, it will cause not let the authorities group work in next
        /// session.
        ClearSwitchFlag,
        /// Disable the authority, and remove it in next session.
        DisableAuthority { account_id: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The new authority group is same as current one, we do nothing for this.
        SameAuthorityGroup,
        /// The new authority group can be empty.
        EmptyAuthorityGroup,
        /// The new authority group contains duplicated authority.
        DuplicatedAuthority,
        /// Disable an authority failed for the authority is not existed or already disabled, or
        /// the authorities only has one so we can not disable it.
        DisableAuthorityFailed,
        /// The account_id has not registered as the validator yet.
        NotAuthority,
    }

    #[pallet::storage]
    #[pallet::getter(fn authorities)]
    pub type Authorities<T: Config> = StorageValue<_, Vec<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn switch_new_group)]
    pub type SwitchNewGroup<T> = StorageValue<_, bool>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub keys: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { keys: Default::default() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if !self.keys.is_empty() {
                Pallet::<T>::set_authorities(self.keys.clone())
                    .expect("Chain spec must be valid for keys.");
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set new authorities group, and this group will be commit to the consensus system in next
        /// session.
        #[pallet::weight(
            (42_131_000 as Weight)
                .saturating_add(T::DbWeight::get().reads(2 as Weight))
                .saturating_add(T::DbWeight::get().writes(1 as Weight))
        )]
        pub fn new_authorities(
            origin: OriginFor<T>,
            authorities: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // check all account_id is registered as validator.
            for a in authorities.iter() {
                if let Ok(validator) = a.clone().try_into() {
                    use frame_support::traits::ValidatorRegistration;
                    ensure!(
                        pallet_session::Pallet::<T>::is_registered(&validator),
                        Error::<T>::NotAuthority
                    );
                }
            }

            Self::set_authorities(authorities)?;
            // set switch flag to trigger AuthorityChange in next session.
            SwitchNewGroup::<T>::put(true);

            // we use the decoded data from the storage, not use the one from parameter, for
            // `set_authorities` does not return result, and we wanna know the processed result.
            if let Some(authorities) = Self::authorities() {
                Self::deposit_event(Event::<T>::PrepareNewAuthorities { group: authorities });
            }
            Ok(())
        }

        /// Disable an authority in current session, and remove it in next session.
        #[pallet::weight(
            (42_131_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        )]
        pub fn disable_authority(origin: OriginFor<T>, authority: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            let validator_id =
                authority.clone().try_into().map_err(|_| Error::<T>::DisableAuthorityFailed)?;
            // disable this authority in current consensus system.
            let result = pallet_session::Pallet::<T>::disable(&validator_id);
            ensure!(result, Error::<T>::DisableAuthorityFailed);

            Authorities::<T>::try_mutate::<_, Error<T>, _>(|current| {
                if let Some(auths) = current {
                    let before_len = auths.len();
                    ensure!(before_len > 1, Error::<T>::DisableAuthorityFailed);
                    // remove the authority in current authority group.
                    auths.retain(|x| x != &authority);

                    let after_len = auths.len();
                    if before_len == after_len {
                        log::warn!(target: TARGET, "The authority does not belong to current authority group, but still disabled in consensus system. now group:{:?}", auths);
                    }
                }
                Ok(())
            })?;
            // set switch flag to trigger AuthorityChange in next session.
            SwitchNewGroup::<T>::put(true);
            Self::deposit_event(Event::DisableAuthority { account_id: authority });

            // force rotate session.
            pallet_session::Pallet::<T>::rotate_session();
            // rotate twice to let the new authorities group take effect immediately
            pallet_session::Pallet::<T>::rotate_session();
            Ok(())
        }

        /// Force to switch authorities group for current authorities storage in next session,
        /// even there is no change in current group.
        #[pallet::weight(42_131_000 as Weight)]
        pub fn force_switch(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            // set flag for switch.
            SwitchNewGroup::<T>::put(true);
            Self::deposit_event(Event::<T>::ForceSwitchGroup);
            Ok(())
        }

        /// Clear switch authorities group flag and authorities will not changed in next session.
        #[pallet::weight(42_131_000 as Weight)]
        pub fn clear_switch(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            // clear flag for switch.
            SwitchNewGroup::<T>::take();
            Self::deposit_event(Event::<T>::ClearSwitchFlag);
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn set_authorities(authorities: Vec<T::AccountId>) -> sp_runtime::DispatchResult {
        let mut authorities = authorities;
        authorities.sort();

        ensure!(!authorities.is_empty(), Error::<T>::EmptyAuthorityGroup);

        let mut iter = authorities.iter();
        let mut last = iter.next().expect("At least contains one element.");
        for n in iter {
            // check duplicated authority.
            if last != n {
                last = n
            } else {
                return Err(Error::<T>::DuplicatedAuthority.into())
            }
        }

        Authorities::<T>::try_mutate::<_, Error<T>, _>(|current| {
            if let Some(old) = current {
                // check new group is same as old group.
                ensure!(*old != authorities, Error::<T>::SameAuthorityGroup);
            }
            // set to new group;
            *current = Some(authorities);
            Ok(())
        })?;
        Ok(())
    }
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
    fn new_session(index: sp_staking::SessionIndex) -> Option<Vec<T::AccountId>> {
        let need_switch = Self::switch_new_group().unwrap_or(false);
        log::debug!(
            target: TARGET,
            "in new_session:{:?} need switch to new group:{:}",
            index,
            need_switch
        );
        if !need_switch {
            return None
        }

        let authorities = Self::authorities();
        if let Some(ref auths) = authorities {
            log::debug!(target: TARGET, "new authorities :{:?}", auths);
            Self::deposit_event(Event::<T>::ChangeAuthorities { index, authorities: auths.clone() })
        } else {
            log::debug!(target: TARGET, "current authorities is `None` and do not change.");
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
