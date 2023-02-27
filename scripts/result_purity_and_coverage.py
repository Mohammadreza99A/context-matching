from pandas import read_csv, DataFrame
import matplotlib.pyplot as plt
import os
import glob
from sys import argv
import numpy as np


def calculate_purity(obs_df, res_df):
    purity = 0

    segments_dict = {}

    segments = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
        1)).cumsum())

    segments_count = segments.iloc[len(segments) - 1]["segment_id"]

    obs_go_fishing_indices = (obs_df[obs_df['label'] == "01-sailing"].iloc[0].name,
                              obs_df[obs_df['label'] == "02-fishing"].iloc[0].name)
    obs_fishing_indices = (obs_df[obs_df['label'] == "02-fishing"].iloc[0].name,
                           obs_df[obs_df['label'] == "03-sailing"].iloc[0].name)
    obs_go_to_port_indices = (obs_fishing_indices[1], len(obs_df))

    row_counter = 0
    for i in range(1, segments_count + 1):
        segment_i = segments[segments["segment_id"] == i]
        correct_labels_count = 0

        segment_i_label = segment_i.iloc[0]["context"]

        for _ in range(0, len(segment_i)):
            if (row_counter >= obs_go_fishing_indices[0] and row_counter < obs_go_fishing_indices[1]):
                if (segment_i_label == "GoFishing"):
                    correct_labels_count += 1
            if (row_counter >= obs_fishing_indices[0] and row_counter < obs_fishing_indices[1]):
                if (segment_i_label == "Fishing"):
                    correct_labels_count += 1
            if (row_counter >= obs_go_to_port_indices[0] and row_counter < obs_go_to_port_indices[1]):
                if (segment_i_label == "GoToPort"):
                    correct_labels_count += 1

            row_counter += 1

        if correct_labels_count != 0:
            purity += correct_labels_count / len(segment_i)
            segments_dict[i] = [correct_labels_count,
                                correct_labels_count / len(segment_i)]

    return purity / len(segments_dict)


def calculate_coverage(obs_df, res_df):
    coverage = 0

    segments = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
        1)).cumsum())

    total_obs_segments_count = 3

    fishing_indices = (obs_df[obs_df['label'] == "02-fishing"].iloc[0].name,
                       obs_df[obs_df['label'] == "03-sailing"].iloc[0].name)

    fishing_segment = segments.iloc[fishing_indices[0]:fishing_indices[1]]
    fishing_segment_correct_labels = len(
        fishing_segment[fishing_segment["context"] == "Fishing"])
    coverage += fishing_segment_correct_labels / len(fishing_segment)

    go_fishing_segment = segments.iloc[0:fishing_indices[0]]
    go_fishing_correct_labels = len(
        go_fishing_segment[go_fishing_segment["context"] == "GoFishing"])
    coverage += go_fishing_correct_labels / len(go_fishing_segment)

    go_to_port_segment = segments.iloc[fishing_indices[1]:len(segments)]
    go_to_port_segment_correct_labels = len(
        go_to_port_segment[go_to_port_segment["context"] == "GoToPort"])
    coverage += go_to_port_segment_correct_labels / len(go_to_port_segment)

    return coverage / total_obs_segments_count


def calculate_harmonic_mean(purity, coverage):
    return (2*purity*coverage) / (purity + coverage)


if __name__ == "__main__":
    if (len(argv) != 3):
        print("Bad number of arguments: <input_obs_folder> <input_res_folder>")
        exit()

    INPUT_OBS_FOLDER = argv[1]
    INPUT_RES_FOLDER = argv[2]

    all_res_files = glob.glob(os.path.join(INPUT_RES_FOLDER, "*.csv"))

    total_purity = 0
    total_coverage = 0
    total_hormonic_mean = 0

    purities = []
    coverages = []
    harmonic_means = []

    for i in range(len(all_res_files)):
        # for i in range(10):
        # file_path = "AIS_trajs/res/219001624-2.csv"
        file_path = all_res_files[i]

        file_name = os.path.basename(file_path)

        obs_df = read_csv(os.path.join(
            INPUT_OBS_FOLDER, file_name), delimiter=",")
        res_df = read_csv(file_path, delimiter=",")

        purity = calculate_purity(obs_df, res_df)
        coverage = calculate_coverage(obs_df, res_df)
        harmonic_mean = calculate_harmonic_mean(purity, coverage)
        purities.append(purity * 100)
        coverages.append(coverage * 100)
        harmonic_means.append(harmonic_mean * 100)
        total_purity += purity
        total_coverage += coverage
        total_hormonic_mean += harmonic_mean

    total_purity = total_purity / len(all_res_files)
    total_coverage = total_coverage / len(all_res_files)
    total_hormonic_mean = total_hormonic_mean / len(all_res_files)

    print("Total purity is {}%".format(total_purity * 100))
    print("Total coverage is {}%".format(total_coverage * 100))
    print("Total harmonic mean is {}%".format(total_hormonic_mean * 100))

    print("min purity is {} and max is {}".format(min(purities), max(purities)))
    print("min coverage is {} and max is {}".format(
        min(coverages), max(coverages)))
    print("min harmonic is {} and max is {}".format(
        min(harmonic_means), max(harmonic_means)))

    df = DataFrame({"purities": purities, "coverages": coverages,
                   "harmonic means": harmonic_means})
    df.plot.density()
    # df.plot.hist()
    font = {'size': 15}
    plt.rc('font', **font)
    plt.rc('xtick', labelsize=18)
    plt.rc('ytick', labelsize=18)
    plt.rc('axes', labelsize=18)

    plt.legend(loc="upper left")
    plt.xticks(np.arange(35, 100+1, 5.0))
    plt.yticks(np.arange(0, 1, 0.1))
    plt.xlim(60, 102)
    plt.show()
