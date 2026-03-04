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

    pub fn process_input(
        &mut self,
        context: &FormalContext<String>,
        input: ExplorationInput,
    ) -> ExplorationState {
        match (&self.state, input) {
            (_, ExplorationInput::Stop) => {
                self.reset();
                ExplorationState::Idle
            }
            (ExplorationState::Idle, ExplorationInput::Start) => self.start_exploration(context),
            (ExplorationState::ValidatingImplication { .. }, ExplorationInput::Yes) => {
                self.handle_yes()
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
            ) => self.handle_submit(context, counterexample),
            (ExplorationState::Finished, ExplorationInput::Start) => {
                self.reset();
                self.start_exploration(context)
            }
            _ => self.state.clone(),
        }
    }

    fn start_exploration(&mut self, context: &FormalContext<String>) -> ExplorationState {
        let all_attributes: BitSet = (0..context.attributes.len()).collect();
        self.temp_set = BitSet::new();
        self.temp_hull = BitSet::new();

        self.explore_next(context, all_attributes)
    }

    fn explore_next(
        &mut self,
        context: &FormalContext<String>,
        all_attributes: BitSet,
    ) -> ExplorationState {
        loop {
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
                    canonical_basis::index_next_preclosure(context, &self.basis, &self.temp_set);
            }
        }
    }

    fn handle_yes(&mut self) -> ExplorationState {
        if let ExplorationState::ValidatingImplication {
            premise,
            conclusion,
        } = &self.state
        {
            self.basis.push((premise.clone(), conclusion.clone()));
            let all_attributes: BitSet = (0..premise.len() + conclusion.len()).collect();
            self.explore_next_from_hull(all_attributes)
        } else {
            self.state.clone()
        }
    }

    fn explore_next_from_hull(&mut self, all_attributes: BitSet) -> ExplorationState {
        loop {
            if self.temp_set == all_attributes {
                self.state = ExplorationState::Finished;
                return ExplorationState::Finished;
            }

            self.temp_hull = self.temp_set.clone();

            if self.temp_set != self.temp_hull {
                self.state = ExplorationState::ValidatingImplication {
                    premise: self.temp_set.clone(),
                    conclusion: self.temp_hull.clone(),
                };
                return self.state.clone();
            } else {
                return ExplorationState::Idle;
            }
        }
    }

    fn handle_submit(
        &mut self,
        context: &FormalContext<String>,
        counterexample: BitSet,
    ) -> ExplorationState {
        self.temp_set = counterexample;
        self.explore_next(context, (0..context.attributes.len()).collect())
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
