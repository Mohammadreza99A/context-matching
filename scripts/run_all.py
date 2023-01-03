from pandas import read_csv
import matplotlib.pyplot as plt
import os
import glob
from sys import argv
import subprocess

if (len(argv) != 3):
    print("Bad number of arguments: <input_folder> <output_folder>")
    exit()

INPUT_FOLDER = argv[1]
OUTPUT_FOLDER = argv[2]


all_files = glob.glob(os.path.join(INPUT_FOLDER, "*.csv"))

for file_path in all_files:
    file_name = os.path.basename(file_path)
    # Execute rust code to generate the result file
    process = subprocess.Popen(
        ['cargo', 'run', '--', file_path, os.path.join(OUTPUT_FOLDER, file_name)])
