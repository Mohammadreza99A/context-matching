from pandas import read_csv
import matplotlib.pyplot as plt
from sys import argv


if (len(argv) != 4):
    print("Bad number of arguments: <result_file> <observation_file> <traj|ctx>")
    exit()

RES_FILE_NAME = argv[1]
OBS_FILE_NAME = argv[2]
PLOT_DESC = argv[3]


def plot_traj(obs_df, res_df):
    obs_x = obs_df["x"].values.tolist()
    obs_y = obs_df["y"].values.tolist()
    res_x = res_df["x"].values.tolist()
    res_y = res_df["y"].values.tolist()

    plt.scatter(obs_x, obs_y, c="b", s=40, label="observations")
    plt.scatter(res_x, res_y, c="r", s=10, label="results")
    plt.legend(loc="upper left")


def plot_ctx(obs_df, res_df):
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

    plt.scatter(obs_sailing_x, obs_sailing_y, c="blue", s=70, label="sailing")
    plt.scatter(obs_fishing_x, obs_fishing_y, c="red", s=70, label="fishing")
    plt.scatter(res_sailing_x, res_sailing_y, c="orange",
                s=30, label="matched sailing")
    plt.scatter(res_fishing_x, res_fishing_y, c="green",
                s=30, label="matched fishing")


if __name__ == "__main__":
    # Read csv files
    obs_df = read_csv(OBS_FILE_NAME, delimiter=",")
    res_df = read_csv(RES_FILE_NAME, delimiter=",")

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

    print("Correct matches: {} --- Incorrect matches: {}".format(correct, false))

    if (PLOT_DESC == "traj"):
        plot_traj(obs_df, res_df)
        plt.legend(loc="upper left")
        plt.show()
    elif (PLOT_DESC == "ctx"):
        plot_ctx(obs_df, res_df)
        # plt.legend(loc="upper left")
        plt.show()
    else:
        print("Description {} is not known.".format(PLOT_DESC))
        exit()
