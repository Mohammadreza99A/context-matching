import pandas as pd
from sys import argv

if (not len(argv) == 3):
    print("Bad number of arguments: <input_file> <output_directory>")
    exit()

INPUT_FILE: str = argv[1]
OUTPUT_DIR: str = argv[2]

df = pd.read_csv(INPUT_FILE)

for (id), group in df.groupby(["id"]):
    group.to_csv(f'{OUTPUT_DIR}{id}.csv', index=False)
