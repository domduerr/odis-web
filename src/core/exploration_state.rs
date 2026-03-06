use bit_set::BitSet;
use odis::algorithms::canonical_basis;
use odis::FormalContext;

#[derive(Debug, Clone, PartialEq)]
pub enum ExplorationState {
    Idle,
    ValidatingImplication { premise: BitSet, conclusion: BitSet },
    AwaitingCounterexample { premise: BitSet, conclusion: BitSet },
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExplorationInput {
    Start,
    Yes,
    No,
    Submit { counterexample: BitSet },
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplorationMachine {
    pub state: ExplorationState,
    pub basis: Vec<(BitSet, BitSet)>,
    pub temp_set: BitSet,
    pub temp_hull: BitSet,
}

impl ExplorationMachine {
    pub fn new() -> Self {
        ExplorationMachine {
            state: ExplorationState::Idle,
            basis: Vec::new(),
            temp_set: BitSet::new(),
            temp_hull: BitSet::new(),
        }
    }

    pub fn process_input<F>(
        &mut self,
        context_getter: F,
        input: ExplorationInput,
    ) -> ExplorationState
    where
        F: Fn() -> FormalContext<String>,
    {
        match (&self.state, input) {
            (_, ExplorationInput::Stop) => {
                self.reset();
                ExplorationState::Idle
            }
            (ExplorationState::Idle, ExplorationInput::Start) => {
                self.start_exploration(context_getter)
            }
            (ExplorationState::ValidatingImplication { .. }, ExplorationInput::Yes) => {
                self.handle_yes(context_getter)
            }
            (
                ExplorationState::ValidatingImplication {
                    premise,
                    conclusion,
                },
                ExplorationInput::No,
            ) => {
                self.state = ExplorationState::AwaitingCounterexample {
                    premise: premise.clone(),
                    conclusion: conclusion.clone(),
                };
                self.state.clone()
            }
            (
                ExplorationState::AwaitingCounterexample { .. },
                ExplorationInput::Submit { counterexample },
            ) => self.handle_submit(context_getter, counterexample),
            (ExplorationState::Finished, ExplorationInput::Start) => {
                self.reset();
                self.start_exploration(context_getter)
            }
            _ => self.state.clone(),
        }
    }

    fn start_exploration<F>(&mut self, context_getter: F) -> ExplorationState
    where
        F: Fn() -> FormalContext<String>,
    {
        let context = context_getter();
        let all_attributes: BitSet = (0..context.attributes.len()).collect();
        self.temp_set = BitSet::new();
        self.temp_hull = BitSet::new();

        self.explore_next(context_getter, all_attributes)
    }

    fn explore_next<F>(&mut self, context_getter: F, all_attributes: BitSet) -> ExplorationState
    where
        F: Fn() -> FormalContext<String>,
    {
        loop {
            let context = context_getter();

            if self.temp_set == all_attributes {
                self.state = ExplorationState::Finished;
                return ExplorationState::Finished;
            }

            self.temp_hull = context.index_attribute_hull(&self.temp_set);

            if self.temp_set != self.temp_hull {
                self.state = ExplorationState::ValidatingImplication {
                    premise: self.temp_set.clone(),
                    conclusion: self.temp_hull.clone(),
                };
                return self.state.clone();
            } else {
                self.temp_set =
                    canonical_basis::index_next_preclosure(&context, &self.basis, &self.temp_set);
            }
        }
    }

    fn handle_yes<F>(&mut self, context_getter: F) -> ExplorationState
    where
        F: Fn() -> FormalContext<String>,
    {
        if let ExplorationState::ValidatingImplication {
            premise: _,
            conclusion: _,
        } = &self.state
        {
            self.basis
                .push((self.temp_set.clone(), self.temp_hull.clone()));
            let context = context_getter();
            self.temp_set =
                canonical_basis::index_next_preclosure(&context, &self.basis, &self.temp_set);
            self.explore_next(context_getter, (0..context.attributes.len()).collect())
        } else {
            self.state.clone()
        }
    }

    fn handle_submit<F>(&mut self, context_getter: F, _counterexample: BitSet) -> ExplorationState
    where
        F: Fn() -> FormalContext<String>,
    {
        let context = context_getter();
        let all_attributes: BitSet = (0..context.attributes.len()).collect();
        self.explore_next(context_getter, all_attributes)
    }

    pub fn reset(&mut self) {
        self.state = ExplorationState::Idle;
        self.basis = Vec::new();
        self.temp_set = BitSet::new();
        self.temp_hull = BitSet::new();
    }
}

impl Default for ExplorationState {
    fn default() -> Self {
        ExplorationState::Idle
    }
}
