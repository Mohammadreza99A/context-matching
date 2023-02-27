from pandas import read_csv
import matplotlib.pyplot as plt
import os
import glob
from sys import argv
from plot import plot_ctx


def plot_ctx_file(obs_df, res_df, file_name):

    plt.figure()

    plot_ctx(obs_df, res_df)

    plt.legend()

    plt.savefig(os.path.join(OUTPUT_FOLDER,
                os.path.splitext(file_name)[0] + ".jpg"), dpi=500)

    plt.close()


if __name__ == "__main__":
    if (len(argv) != 4):
        print(
            "Bad number of arguments: <input_obs_folder> <input_res_folder> <output_folder>")
        exit()

    INPUT_OBS_FOLDER = argv[1]
    INPUT_RES_FOLDER = argv[2]
    OUTPUT_FOLDER = argv[3]

    all_res_files = glob.glob(os.path.join(INPUT_RES_FOLDER, "*.csv"))

    for file_path in all_res_files:
        file_name = os.path.basename(file_path)

        obs_df = read_csv(os.path.join(
            INPUT_OBS_FOLDER, file_name), delimiter=",")
        res_df = read_csv(file_path, delimiter=",")

        plot_ctx_file(obs_df, res_df, file_name)
