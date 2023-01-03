from pandas import read_csv
import matplotlib.pyplot as plt
import os
import glob
from sys import argv

if (len(argv) != 3):
    print("Bad number of arguments: <input_obs_folder> <input_res_folder>")
    exit()

INPUT_OBS_FOLDER = argv[1]
INPUT_RES_FOLDER = argv[2]

total_process_files = 0
success_rate_sum = 0

all_res_files = glob.glob(os.path.join(INPUT_RES_FOLDER, "*.csv"))

for file_path in all_res_files:
    file_name = os.path.basename(file_path)

    obs_df = read_csv(os.path.join(INPUT_OBS_FOLDER, file_name), delimiter=",")
    res_df = read_csv(file_path, delimiter=",")

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

    total_process_files += 1
    success_rate_sum += correct / (correct + false)

print("Average success rate for {} processed files is {}".format(
    total_process_files, success_rate_sum / total_process_files))
