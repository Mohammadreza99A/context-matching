from sys import argv
import pandas as pd
import numpy as np
import plotly.graph_objs as go
import plotly.subplots as sp
import dash
from dash import dcc, html
from dash.dependencies import Input, Output


def generate_plot(history_file, obs_file):

    hist_df = pd.read_csv(history_file)
    obs_df = pd.read_csv(obs_file)

    # Group the rows by id and create a dictionary of DataFrames
    hist_df_dict = {k: v for k, v in hist_df.groupby('t')}

    # Define the color map
    color_map = {
        'GoFishing': 'green',
        'Fishing': 'red',
        'GoToPort': 'blue',
        '01-sailing': 'green',
        '02-fishing': 'red',
        '03-sailing': 'blue',
    }

    legend_map = {
        'GoFishing': 'green',
        'Fishing': 'red',
        'GoToPort': 'blue',
    }

    def get_trace(df, col):
        trace = go.Scatter(
            x=list(range(1, list(hist_df_dict.keys())[-1] + 1)),
            # x=list(range(1, len(df)+1)),
            y=[int(col[2:])]*len(df),
            mode='markers',
            marker=dict(
                size=5,
                color=df[col].map(color_map)
            ),
            showlegend=False
        )
        return trace

    def get_bar_trace(df):
        bar_trace = go.Scatter(
            x=df.index+1,
            y=np.zeros(len(df)),
            mode='markers',
            marker=dict(
                size=10,
                color=df['label'].map(color_map)
            ),
            showlegend=False

        )
        return bar_trace

    def get_legend_data():
        legend_data = []
        for key, value in legend_map.items():
            legend_data.append(
                go.Scatter(
                    x=[None],
                    y=[None],
                    mode='markers',
                    marker=dict(
                        size=10,
                        color=value
                    ),
                    name=key
                )
            )
        return legend_data

    # create the initial plot
    df = hist_df_dict[0]
    traces = []
    for col in df.columns:
        if col.startswith('p_'):
            traces.append(get_trace(df, col))

    # create the bar plot trace
    bar_trace = get_bar_trace(obs_df)

    # create the legend data
    legend_data = get_legend_data()

    # create the subplot
    fig = sp.make_subplots(
        rows=2, cols=1, subplot_titles=("Particles", "Observations"))
    fig.add_traces(traces, rows=1, cols=1)
    fig.add_trace(bar_trace, row=2, col=1)
    fig.add_traces(legend_data)

    # update the layout
    fig.update_layout(
        title='Context-Matching Debugging Tool',
        xaxis=dict(
            title='History',
            range=[0, len(hist_df_dict) + 3]
        ),
        yaxis=dict(
            title='Particles'
        ),
        xaxis2=dict(
            title=''
        ),
        yaxis2=dict(
            showticklabels=False
        ),
    )

    # Marks for the slider
    slider_marks = {}
    i = 0
    for k in hist_df_dict:
        slider_marks[i] = str(k)
        i += 1

    # create the app and layout
    app = dash.Dash(__name__)
    app.layout = html.Div([
        dcc.Graph(id='graph', figure=fig),
        html.Div(id='chosen-slider-value', style={'margin-bottom': 20}),
        dcc.Slider(
            id='slider',
            min=list(slider_marks.keys())[0] - 1,
            max=list(slider_marks.keys())[-1] + 1,
            step=1,
            value=hist_df['t'][0],
            marks=slider_marks,
        )
    ])

    @app.callback(Output('chosen-slider-value', 'children'),
                  Input('slider', 'value'))
    def display_slider_value(value):
        return 'Chosen time: {}'.format(int(slider_marks[value]))

    # create the callback function to update the plot based on the slider value
    @app.callback(Output('graph', 'figure'), [Input('slider', 'value')])
    def update_figure(selected_value):
        selected_value = int(slider_marks[selected_value])

        print(selected_value)

        df = hist_df_dict[selected_value]

        # create the scatter plot trace
        traces = []
        for col in df.columns:
            if col.startswith('p_'):
                traces.append(get_trace(df, col))

        # create the bar plot trace
        bar_trace = get_bar_trace(obs_df)

        # create the legend data
        legend_data = get_legend_data()

        # update the subplot
        fig = sp.make_subplots(
            rows=2, cols=1, subplot_titles=("Particles", "Observations"), row_heights=[400, 15])
        fig.add_traces(traces, rows=1, cols=1)
        fig.add_trace(bar_trace, row=2, col=1)
        fig.add_traces(legend_data)
        fig.update_layout(width=1800, height=1024)

        fig.update_layout(
            title='Context-Matching Debugging Tool',
            xaxis=dict(
                title='History',
                range=[0, len(hist_df_dict) + 3]
            ),
            yaxis=dict(
                title='Particles'
            ),
            xaxis2=dict(
                title='Observation'
            ),
            yaxis2=dict(
                showticklabels=False
            ),
        )

        return fig

    app.run_server(debug=True)


if __name__ == "__main__":
    if (len(argv) != 3):
        print(
            "Bad number of arguments: <input_history_file> <obs_file>")
        exit()

    INPUT_HISTORY_FILE = argv[1]
    OBS_FILE = argv[2]

    history_df = generate_plot(INPUT_HISTORY_FILE, OBS_FILE)
