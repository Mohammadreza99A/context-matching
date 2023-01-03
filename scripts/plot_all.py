from pandas import read_csv
import matplotlib.pyplot as plt
import os
import glob
from sys import argv

if (len(argv) != 4):
    print("Bad number of arguments: <input_obs_folder> <input_res_folder> <output_folder>")
    exit()

INPUT_OBS_FOLDER = argv[1]
INPUT_RES_FOLDER = argv[2]
OUTPUT_FOLDER = argv[3]


def plot_ctx(obs_df, res_df, file_name):
    obs_x = obs_df["x"].values.tolist()
    obs_y = obs_df["y"].values.tolist()
    res_context = res_df["context"].values.tolist()

    obs_sailing_x = obs_df[obs_df['label'].str.contains("sailing")]["x"]
    obs_sailing_y = obs_df[obs_df['label'].str.contains("sailing")]["y"]
    obs_fishing_x = obs_df[obs_df['label'].str.contains("fishing")]["x"]
    obs_fishing_y = obs_df[obs_df['label'].str.contains("fishing")]["y"]

    res_sailing_x = []
    res_sailing_y = []
    res_fishing_x = []
    res_fishing_y = []

    for i in range(len(res_context)):
        if (res_context[i] == "SAILING"):
            res_sailing_x.append(obs_x[i])
            res_sailing_y.append(obs_y[i])
        if (res_context[i] == "FISHING"):
            res_fishing_x.append(obs_x[i])
            res_fishing_y.append(obs_y[i])

    correct = 0
    false = 0
    obs_context = obs_df["label"].values.tolist()
    res_context = res_df["context"].values.tolist()

    for i in range(len(res_context)):
        if (res_context[i] == "FISHING" and "fishing" in obs_context[i]):
            correct += 1
        elif (res_context[i] == "SAILING" and "sailing" in obs_context[i]):
            correct += 1
        else:
            false += 1

    plt.figure()

    plt.title("Correct: {} --- Incorrect: {} --- Success rate: {}".format(correct,
              false, correct / (correct + false)))

    plt.scatter(obs_sailing_x, obs_sailing_y, c="blue", s=70, label="sailing")
    plt.scatter(obs_fishing_x, obs_fishing_y, c="red", s=70, label="fishing")
    plt.scatter(res_sailing_x, res_sailing_y, c="orange",
                s=30, label="matched sailing")
    plt.scatter(res_fishing_x, res_fishing_y, c="green",
                s=30, label="matched fishing")

    plt.legend(loc="upper left")

    plt.savefig(os.path.join(OUTPUT_FOLDER,
                os.path.splitext(file_name)[0] + ".jpg"))

    plt.close()


all_res_files = glob.glob(os.path.join(INPUT_RES_FOLDER, "*.csv"))

for file_path in all_res_files:
    file_name = os.path.basename(file_path)

    obs_df = read_csv(os.path.join(INPUT_OBS_FOLDER, file_name), delimiter=",")
    res_df = read_csv(file_path, delimiter=",")

    plot_ctx(obs_df, res_df, file_name)
