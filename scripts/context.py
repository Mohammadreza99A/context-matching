import math
from shapely.geometry import Point
from dataclasses import dataclass
from enum import Enum
import numpy as np
from pandas import read_csv
import matplotlib.pyplot as plt
from sys import argv
import time


def generate_random_point() -> Point:
    return Point(np.random.uniform(), np.random.uniform())


class ContextType(Enum):
    SAILING = 0
    FISHING = 1


@dataclass
class ContextState:
    pos: Point
    dir: Point
    heading: float  # in degree
    speed: float
    context: ContextType


@dataclass
class Observation:
    pos: Point
    time: int
    # just for comparing at the end. this context is not used in the particle filtering
    real_context: ContextType

    @staticmethod
    def read_observation_csv(filename: str) -> list['Observation']:
        df = read_csv(filename, delimiter=",")

        observations: list[Observation] = [Observation(
            Point(o.x, o.y),
            o.time_gap,
            ContextType.FISHING
            if "fishing" in o.label
            else ContextType.SAILING)
            for o in df.itertuples()]

        return observations


class FishingContext:
    observations: list[Observation]
    nb_of_particles: int
    sigma: float
    alpha: float

    def __init__(self,
                 observations: list[Observation],
                 number_of_particles: int,
                 sigma: float,
                 alpha: float) -> None:
        self.observations = observations
        self.nb_of_particles = number_of_particles
        self.sigma = sigma
        self.alpha = alpha

    def particle_filter(self) -> list[ContextState]:
        res_states: list[ContextState] = []
        samples: list[ContextState] = []

        # Generate first sample
        for _ in range(self.nb_of_particles):
            direction: Point = generate_random_point()
            heading: float = np.random.uniform(0, 360.0)
            state: ContextState = ContextState(
                self.observations[0].pos, direction, heading, 0,
                ContextType.SAILING)
            updated_state: ContextState = self.update(
                state, self.observations[1].time)
            samples.append(updated_state)

        # Generate samples for each observation
        for i in range(len(self.observations)-1):
            samples = self.updates(samples, self.observations[i+1].time)
            weights: list[float] = self.importance_sampling(
                samples, self.observations[i+1])
            max_arg_index: int = weights.index(max(weights))
            res_states.append(samples[max_arg_index])
            samples = self.resample(samples, weights)
        return res_states

    def updates(self,
                samples: list[ContextState],
                time_diff: int) -> list[ContextState]:
        new_samples: list[ContextState] = []

        for i in range(len(samples)):
            new_samples.append(self.update(samples[i], time_diff))

        return new_samples

    def update(self, state: ContextState, time_diff: int) -> ContextState:
        # Update heading:
        # randomly  (uniform) chosen from -0.4 to +0.4 radians (appx 22.91
        # degree) from previous heading
        new_heading: float = np.random.uniform(
            state.heading - 22.91, state.heading + 22.91)

        # Update context:
        # 10% chance that the context changes to the other one
        context_change_prob: float = np.random.uniform()
        new_context: ContextType = state.context
        if (context_change_prob <= 0.1):
            if (state.context == ContextType.FISHING):
                new_context = ContextType.SAILING
            else:
                new_context = ContextType.FISHING

        # Update speed
        # The speed is set according to the context
        new_speed = state.speed
        if (new_context == ContextType.FISHING):
            new_speed = np.random.normal(1.36, 0.89)
        else:
            new_speed = np.random.normal(3.31, 1.19)

        # Update position
        new_dir_x: float = state.dir.x - 0.4 + 2.0 * np.random.uniform() * 0.4
        new_dir_y: float = state.dir.y - 0.4 + 2.0 * np.random.uniform() * 0.4
        new_dir: Point = Point(new_dir_x, new_dir_y)
        norm: float = 1 / self.norm(new_dir)
        new_dir = Point(new_dir.x * norm, new_dir.y * norm)
        new_pos_x: float = state.pos.x + (new_speed * time_diff * new_dir.x)
        new_pos_y: float = state.pos.y + (new_speed * time_diff * new_dir.y)
        new_pos: Point = Point(new_pos_x, new_pos_y)

        return ContextState(new_pos, new_dir, new_heading,
                            new_speed, new_context)

    def importance_sampling(self, states: list[ContextState],
                            observation: Observation) -> list[float]:
        weighted_samples: list[float] = []

        for state in states:
            weighted_samples.append(
                self.alpha * self.calc_emission_prob(observation, state))

        return weighted_samples

    def resample(self, states: list[ContextState],
                 weights: list[float]) -> list[ContextState]:
        T: float = 0
        new_samples: list[ContextState] = []
        k: list[float] = [0 for _ in range(len(weights))]

        for i in range(len(weights)):
            T += weights[i]
            k[i] = T

        for i in range(self.nb_of_particles):
            t: float = np.random.uniform(0, T)

            j: int = 0
            while (k[j] < t):
                j += 1
            new_samples.append(states[j])

        return new_samples

    def calc_emission_prob(self,
                           observation: Observation,
                           state: ContextState) -> float:

        p: Point = Point(observation.pos.x - state.pos.x,
                         observation.pos.y - state.pos.y)
        gc: float = self.norm(p)
        first_term = 1 / (math.sqrt(2 * math.pi) * self.sigma)
        second_term = math.exp(-0.5 * pow((gc / self.sigma), 2))
        # denom: float = math.sqrt(2 * math.pi) * self.sigma
        # gc: float = self.great_circle(
        #     observation.pos.x, observation.pos.y, state.pos.x, state.pos.y)
        # num: float = math.exp(-0.5 * pow((gc / self.sigma), 2))
        # return num / denom
        return first_term * second_term

    def great_circle(self, lon1, lat1, lon2, lat2):
        lon1, lat1, lon2, lat2 = map(math.radians, [lon1, lat1, lon2, lat2])
        return 6371 * (
            math.acos(math.sin(lat1) * math.sin(lat2) + math.cos(lat1)
                      * math.cos(lat2) * math.cos(lon1 - lon2))
        )

    def norm(self, point: Point, point2: Point | None = None) -> float:
        if (isinstance(point2, Point)):
            return math.sqrt(pow(point2.x - point.x, 2) + pow(point2.y - point.y, 2))

        return math.sqrt(pow(point.x, 2) + pow(point.y, 2))


if __name__ == "__main__":
    if (len(argv) != 2):
        print("Bad number of arguments: <csv file>")
        exit()

    print("Reading and parsing input CSV file...")
    observations: list[Observation] = Observation.read_observation_csv(argv[1])

    print("Particle filtering...")
    start_time = time.time()
    fishing_ctx = FishingContext(observations, 100, 80.0, 10**5)
    states: list[ContextState] = fishing_ctx.particle_filter()
    end_time = time.time()
    print("Particle filtering took {} seconds".format(end_time - start_time))

    print("Analyzing results...")
    correct: int = 0
    false: int = 0

    for i in range(len(states)):
        if (observations[i].real_context == states[i].context):
            correct += 1
        else:
            false += 1

    print("Prediction: correct {} false {}".format(correct, false))
    print("Length of observations: {} and length of states: {}.".format(
        len(observations), len(states)))

    print("Generating plot of the results...")

    obs_x = [p.pos.x for p in observations]
    obs_y = [p.pos.y for p in observations]
    plt.scatter(obs_x, obs_y, c="b")

    states_x = [s.pos.x for s in states]
    states_y = [s.pos.y for s in states]
    plt.scatter(states_x, states_y, c="r", alpha=0.6)

    plt.show()
