from pandas import read_csv
import matplotlib.pyplot as plt
from sys import argv
from result_purity_and_coverage import calculate_purity, calculate_coverage, calculate_harmonic_mean


def plot_ctx(obs_df, res_df):
    obs_x = obs_df["x"].values.tolist()
    obs_y = obs_df["y"].values.tolist()
    res_context = res_df["context"].values.tolist()

    obs_go_to_fishing_x = obs_df[obs_df['label'].str.contains(
        "01-sailing")]["x"]
    obs_go_to_fishing_y = obs_df[obs_df['label'].str.contains(
        "01-sailing")]["y"]
    obs_fishing_x = obs_df[obs_df['label'].str.contains("fishing")]["x"]
    obs_fishing_y = obs_df[obs_df['label'].str.contains("fishing")]["y"]
    obs_go_to_port_x = obs_df[obs_df['label'].str.contains("03-sailing")]["x"]
    obs_go_to_port_y = obs_df[obs_df['label'].str.contains("03-sailing")]["y"]

    res_go_fishing_x = []
    res_go_fishing_y = []
    res_fishing_x = []
    res_fishing_y = []
    res_go_to_port_x = []
    res_go_to_port_y = []

    for i in range(len(res_context)):
        if (res_context[i] == "GoFishing"):
            res_go_fishing_x.append(obs_x[i])
            res_go_fishing_y.append(obs_y[i])
        if (res_context[i] == "Fishing"):
            res_fishing_x.append(obs_x[i])
            res_fishing_y.append(obs_y[i])
        if (res_context[i] == "GoToPort"):
            res_go_to_port_x.append(obs_x[i])
            res_go_to_port_y.append(obs_y[i])

    plt.scatter(obs_go_to_fishing_x, obs_go_to_fishing_y,
                c="blue", s=70, label="go fishing")
    plt.scatter(obs_fishing_x, obs_fishing_y, c="red", s=70, label="fishing")
    plt.scatter(obs_go_to_port_x, obs_go_to_port_y,
                c="gray", s=70, label="go to port")
    plt.scatter(res_go_to_port_x, res_go_to_port_y, c="cyan",
                s=30, label="matched go to port")
    plt.scatter(res_go_fishing_x, res_go_fishing_y, c="orange",
                s=30, label="matched go fishing")
    plt.scatter(res_fishing_x, res_fishing_y, c="green",
                s=30, label="matched fishing")

    plt.title("Purity = {:.2f}, Coverage = {:.2f}".format(calculate_purity(
        obs_df, res_df), calculate_coverage(obs_df, res_df)))


def print_results(obs_context, res_context):
    correct = 0
    false = 0

    for i in range(len(res_context)):
        if (res_context[i] == "Fishing" and "fishing" in obs_context[i]):
            correct += 1
        elif (res_context[i] == "GoFishing" and "01-sailing" in obs_context[i]):
            correct += 1
        elif (res_context[i] == "GoToPort" and "03-sailing" in obs_context[i]):
            correct += 1
        else:
            false += 1

    print("Correct matches: {} --- Incorrect matches: {}".format(correct, false))


if __name__ == "__main__":
    if (len(argv) != 3):
        print("Bad number of arguments: <result_file> <observation_file>")
        exit()

    RES_FILE_NAME = argv[1]
    OBS_FILE_NAME = argv[2]

    # Read csv files
    obs_df = read_csv(OBS_FILE_NAME, delimiter=",")
    res_df = read_csv(RES_FILE_NAME, delimiter=",")

    # purity = calculate_purity(obs_df, res_df)
    # coverage = calculate_coverage(obs_df, res_df)
    # harmonic_mean = calculate_harmonic_mean(purity, coverage)
    # print("Purity = {} -- Coverage = {} -- Harmonic mean = {}".format(purity,
    #       coverage, harmonic_mean))

    plot_ctx(obs_df, res_df)
    plt.legend()
    plt.show()
