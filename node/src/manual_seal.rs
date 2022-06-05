use std::cell::RefCell;

const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = *b"timstap0";
thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
/// Each call will increment timestamp by slot_duration making Aura think time has passed.
pub struct MockTimestampInherentDataProvider;

#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
    fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        TIMESTAMP.with(|x| {
            *x.borrow_mut() += amax_eva_runtime::constants::time::SLOT_DURATION;
            inherent_data.put_data(INHERENT_IDENTIFIER, &*x.borrow())
        })
    }

    async fn try_handle_error(
        &self,
        _identifier: &sp_inherents::InherentIdentifier,
        _error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        // The pallet never reports error.
        None
    }
}
