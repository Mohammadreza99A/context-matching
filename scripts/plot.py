from pandas import read_csv
import matplotlib.pyplot as plt
from sys import argv


if (len(argv) != 3):
    print("Bad number of arguments: <result_file> <observation_file")
    exit()

RES_FILE_NAME = argv[1]
OBS_FILE_NAME = argv[2]

# Read csv files
obs_df = read_csv(OBS_FILE_NAME, delimiter=",")
res_df = read_csv(RES_FILE_NAME, delimiter=",")


obs_x = obs_df["x"].values.tolist()
obs_y = obs_df["y"].values.tolist()
plt.scatter(obs_x, obs_y, c="b")

res_x = res_df["x"].values.tolist()
res_y = res_df["y"].values.tolist()
plt.scatter(res_x, res_y, c="r", alpha=0.6)

# Plot the result
# obs_df.plot.scatter(x="x", y="y")
# res_df.plot.scatter(x="x", y="y")
plt.show()
