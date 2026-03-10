#[derive(Default)]
pub struct ReadinessState {
    producer_ready: bool,
}

impl ReadinessState {
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.producer_ready
    }

    pub fn mark_ready(&mut self) {
        self.producer_ready = true;
    }

    pub fn mark_unready(&mut self) {
        self.producer_ready = false;
    }
}

#[derive(Default)]
pub struct LivenessState {
    producer_live: bool,
}

impl LivenessState {
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.producer_live
    }

    pub fn mark_live(&mut self) {
        self.producer_live = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod liveness {
        use super::*;

        #[test]
        fn liveness_starts_unlive() {
            let liveness = LivenessState::default();
            assert!(!liveness.is_live());
        }

        #[test]
        fn liveness_can_be_marked_live() {
            let mut liveness = LivenessState::default();
            liveness.mark_live();
            assert!(liveness.is_live());
        }
    }

    mod readiness {
        use super::*;

        #[test]
        fn readiness_starts_unready() {
            let readiness = ReadinessState::default();
            assert!(!readiness.is_ready());
        }

        #[test]
        fn readiness_can_be_marked_ready() {
            let mut readiness = ReadinessState::default();
            readiness.mark_ready();
            assert!(readiness.is_ready());
        }

        #[test]
        fn readiness_can_be_marked_unready() {
            let mut readiness = ReadinessState::default();
            readiness.mark_ready();
            readiness.mark_unready();
            assert!(!readiness.is_ready());
        }

        #[test]
        fn readiness_transitions_ready_to_unready() {
            let mut readiness = ReadinessState::default();
            readiness.mark_ready();
            assert!(readiness.is_ready());

            readiness.mark_unready();
            assert!(!readiness.is_ready());
        }
    }
}
