use super::*;
use crate::algorithms::nsga2::Objective;
use crate::construction::heuristics::InsertionContext;
use crate::models::common::SingleDimLoad;
use crate::models::Problem;
use crate::solver::mutation::*;
use crate::solver::population::{Individual, SelectionPhase};
use crate::solver::RefinementContext;
use crate::utils::{parallel_into_collect, unwrap_from_result, Environment};
use std::cmp::Ordering;
use std::sync::Arc;

/// A type which specifies probability behavior for mutation selection.
pub type MutationProbability = Box<dyn Fn(&RefinementContext, &InsertionContext) -> bool + Send + Sync>;

/// A type which specifies a group of multiple mutation strategies with their probability.
pub type MutationGroup = Vec<(Arc<dyn Mutation + Send + Sync>, MutationProbability)>;

/// A simple hyper-heuristic which selects mutation operator from the list with fixed (static) probabilities.
pub struct StaticSelective {
    mutation_group: MutationGroup,
}

impl HyperHeuristic for StaticSelective {
    fn search(&mut self, refinement_ctx: &RefinementContext, individuals: Vec<&Individual>) -> Vec<Individual> {
        parallel_into_collect(individuals.iter().enumerate().collect(), |(idx, insertion_ctx)| {
            refinement_ctx
                .environment
                .parallelism
                .thread_pool_execute(idx, || self.mutate(refinement_ctx, insertion_ctx))
        })
    }
}

impl StaticSelective {
    /// Creates an instance of `StaticSelective` from mutation groups.
    pub fn new(mutation_group: MutationGroup) -> Self {
        Self { mutation_group }
    }

    /// Creates an instance of `StaticSelective` with default parameters.
    pub fn new_with_defaults(problem: Arc<Problem>, environment: Arc<Environment>) -> Self {
        let default_mutation = Self::create_default_mutation(problem);
        let local_search = Arc::new(LocalSearch::new(Box::new(CompositeLocalOperator::new(
            vec![
                (Box::new(ExchangeInterRouteBest::default()), 100),
                (Box::new(ExchangeInterRouteRandom::default()), 30),
                (Box::new(ExchangeIntraRouteRandom::default()), 30),
            ],
            1,
            2,
        ))));

        Self::new(vec![
            (
                Arc::new(DecomposeSearch::new(default_mutation.clone(), (2, 4), 4)),
                create_context_mutation_probability(
                    300,
                    10,
                    vec![(SelectionPhase::Exploration, 0.01), (SelectionPhase::Exploitation, 0.02)],
                    environment.random.clone(),
                ),
            ),
            (local_search.clone(), create_scalar_mutation_probability(0.05, environment.random.clone())),
            (default_mutation, create_scalar_mutation_probability(1., environment.random.clone())),
            (local_search, create_scalar_mutation_probability(0.05, environment.random.clone())),
        ])
    }

    fn mutate(&self, refinement_ctx: &RefinementContext, insertion_ctx: &InsertionContext) -> InsertionContext {
        unwrap_from_result(
            self.mutation_group.iter().filter(|(_, probability)| probability(refinement_ctx, insertion_ctx)).try_fold(
                insertion_ctx.deep_copy(),
                |ctx, (mutation, _)| {
                    let new_insertion_ctx = mutation.mutate(refinement_ctx, &ctx);

                    if refinement_ctx.problem.objective.total_order(&insertion_ctx, &new_insertion_ctx)
                        == Ordering::Greater
                    {
                        // NOTE exit immediately as we don't want to lose improvement from original individual
                        Err(new_insertion_ctx)
                    } else {
                        Ok(new_insertion_ctx)
                    }
                },
            ),
        )
    }

    /// Creates default mutation (ruin and recreate) with default parameters.
    pub fn create_default_mutation(problem: Arc<Problem>) -> Arc<dyn Mutation + Send + Sync> {
        // initialize recreate
        let recreate = Box::new(CompositeRecreate::new(vec![
            (Box::new(RecreateWithSkipBest::new(1, 2)), 50),
            (Box::new(RecreateWithRegret::new(2, 3)), 20),
            (Box::new(RecreateWithCheapest::default()), 20),
            (Box::new(RecreateWithPerturbation::default()), 10),
            (Box::new(RecreateWithSkipBest::new(3, 4)), 5),
            (Box::new(RecreateWithGaps::default()), 5),
            // TODO use dimension size from problem
            (Box::new(RecreateWithBlinks::<SingleDimLoad>::default()), 5),
            (Box::new(RecreateWithFarthest::default()), 2),
            (Box::new(RecreateWithSkipBest::new(4, 8)), 2),
            (Box::new(RecreateWithNearestNeighbor::default()), 1),
        ]));

        // initialize ruin
        let random_route = Arc::new(RandomRouteRemoval::default());
        let random_job = Arc::new(RandomJobRemoval::new(JobRemovalLimit::default()));
        let ruin = Box::new(CompositeRuin::new(vec![
            (
                vec![
                    (Arc::new(AdjustedStringRemoval::default()), 1.),
                    (Arc::new(NeighbourRemoval::new(JobRemovalLimit::new(2, 8, 0.1))), 0.1),
                    (random_job.clone(), 0.05),
                    (random_route.clone(), 0.01),
                ],
                100,
            ),
            (
                vec![
                    (Arc::new(WorstJobRemoval::default()), 1.),
                    (random_job.clone(), 0.05),
                    (random_route.clone(), 0.01),
                ],
                10,
            ),
            (
                vec![
                    (Arc::new(NeighbourRemoval::default()), 1.),
                    (random_job.clone(), 0.05),
                    (random_route.clone(), 0.01),
                ],
                10,
            ),
            (vec![(random_job.clone(), 1.), (random_route.clone(), 0.1)], 2),
            (vec![(random_route.clone(), 1.), (random_job.clone(), 0.1)], 2),
            (
                vec![
                    (Arc::new(ClusterRemoval::new_with_defaults(problem)), 1.),
                    (random_job, 0.05),
                    (random_route, 0.01),
                ],
                1,
            ),
        ]));

        Arc::new(RuinAndRecreate::new(recreate, ruin))
    }
}

/// Creates a mutation probability which uses `is_hit` method from passed random object.
pub fn create_scalar_mutation_probability(
    scalar_probability: f64,
    random: Arc<dyn Random + Send + Sync>,
) -> MutationProbability {
    Box::new(move |_, _| random.is_hit(scalar_probability))
}

/// Creates a mutation probability which uses context state.
pub fn create_context_mutation_probability(
    jobs_threshold: usize,
    routes_threshold: usize,
    phases: Vec<(SelectionPhase, f64)>,
    random: Arc<dyn Random + Send + Sync>,
) -> MutationProbability {
    let phases = phases.into_iter().collect::<HashMap<_, _>>();
    Box::new(move |refinement_ctx, insertion_ctx| {
        let below_thresholds = insertion_ctx.problem.jobs.size() < jobs_threshold
            || insertion_ctx.solution.routes.len() < routes_threshold;

        if below_thresholds {
            return false;
        }

        let phase_probability = phases.get(&refinement_ctx.population.selection_phase()).cloned().unwrap_or(0.);

        random.is_hit(phase_probability)
    })
}
