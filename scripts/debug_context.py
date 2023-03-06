from sys import argv
import pandas as pd
import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
import numpy as np


def generate_plot(history_file):
    # Generate the dataframe from the input file
    old_df = pd.read_csv(history_file)
    old_df = old_df.rename(columns={"obs_ctx": "p_0"})

    # melt columns p_0 to p_100 into a single column 'p'
    df_melted = old_df.melt(id_vars=['id'], value_vars=[
        f'p_{i}' for i in range(0, 101)], var_name='y', value_name='ctx')
    # set y as integer instead of string
    df_melted['y'] = df_melted['y'].str.extract('(\d+)').astype(int)

    # create a new dataframe with columns ctx, x, y
    df = pd.DataFrame(
        {'ctx': df_melted['ctx'], 'x': df_melted['id'], 'y': df_melted['y']})

    print(df)

    df['range'] = df['x'].apply(lambda x: x//100)

    fig = px.scatter(df, x='x', y='y', color='ctx', animation_frame='range')

    # Set the range of the x-axis to the range of x values for each slider step
    fig.update_xaxes(range=[-1, 101])

    # Update the range of the x-axis for each subsequent slider step
    for i in range(1, df['range'].max() + 1):
        x_min = i * 100
        x_max = (i + 1) * 100 - 1
        fig['frames'][i]['layout']['xaxis']['range'] = [x_min - 1, x_max + 1]

    fig.show()


if __name__ == "__main__":
    if (len(argv) != 2):
        print(
            "Bad number of arguments: <input_history_file>")
        exit()

    INPUT_HISTORY_FILE = argv[1]

    history_df = generate_plot(INPUT_HISTORY_FILE)
