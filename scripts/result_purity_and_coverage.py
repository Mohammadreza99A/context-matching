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


def calculate_purity(obs_df, res_df):
    res_df_segmented = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
        1)).cumsum())

    total_res_segments_count = res_df_segmented.iloc[len(
        res_df_segmented)-1]['segment_id']
    total_obs_segments_count = 3

    obs_df_sailing_1 = obs_df[obs_df['label'] == "01-sailing"]
    obs_df_fishing_2 = obs_df[obs_df['label'] == "02-fishing"]
    obs_df_sailing_3 = obs_df[obs_df['label'] == "03-sailing"]

    obs_segments = [obs_df_sailing_1, obs_df_fishing_2, obs_df_sailing_3]

    purity = 0

    first_seg_index = obs_df_sailing_1.iloc[0].name
    second_seg_index = obs_df_fishing_2.iloc[0].name
    if len(obs_df_sailing_3) > 0:
        third_seg_index = obs_df_sailing_3.iloc[0].name

    res_df_sailing_1 = res_df_segmented.loc[first_seg_index:second_seg_index]
    res_df_fishing_2 = res_df_segmented.loc[second_seg_index +
                                            1: third_seg_index]
    if len(obs_df_sailing_3) > 0:
        res_df_sailing_3 = res_df_segmented.loc[third_seg_index + 1:]

    # for i in range(total_res_segments_count):
    #     segment_i = res_df_segmented[res_df_segmented["segment_id"] == i]
    #     total_segment_points = len(segment_i)

    first_seg_sailing = len(
        res_df_sailing_1[res_df_sailing_1["context"] == "SAILING"])
    first_seg_fishing = len(
        res_df_sailing_1[res_df_sailing_1["context"] == "FISHING"])
    second_seg_sailing = len(
        res_df_fishing_2[res_df_fishing_2["context"] == "SAILING"])
    second_seg_fishing = len(
        res_df_fishing_2[res_df_fishing_2["context"] == "FISHING"])
    third_seg_sailing = len(
        res_df_sailing_3[res_df_sailing_3["context"] == "SAILING"])
    third_seg_fishing = len(
        res_df_sailing_3[res_df_sailing_3["context"] == "FISHING"])

    purity = (max(first_seg_sailing, first_seg_fishing) / len(res_df_sailing_1)) + \
        (max(second_seg_sailing, second_seg_fishing) / len(res_df_fishing_2)) + \
        (max(third_seg_sailing, third_seg_fishing) / len(res_df_sailing_3))
    purity = purity / total_res_segments_count

    print(purity)

    return purity


def calculate_coverage(obs_df, res_df):
    res_df_segmented = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
        1)).cumsum())

    total_res_segments_count = res_df_segmented.iloc[len(
        res_df_segmented)-1]['segment_id']
    total_obs_segments_count = 3

    obs_df_sailing_1 = obs_df[obs_df['label'] == "01-sailing"]
    obs_df_fishing_2 = obs_df[obs_df['label'] == "02-fishing"]
    obs_df_sailing_3 = obs_df[obs_df['label'] == "03-sailing"]

    obs_segments = [obs_df_sailing_1, obs_df_fishing_2, obs_df_sailing_3]

    total_coverage = 0

    for obs_segment in obs_segments:
        segment_context = obs_segment.iloc[0]["label"]
        if ("fishing" in segment_context):
            segment_context = "FISHING"
        else:
            segment_context = "SAILING"

        # Find overlapping segments with this observation segment
        begin_index = obs_segment.iloc[0].name
        end_index = obs_segment.iloc[len(obs_segment)-1].name
        overlapping_res_segments = res_df_segmented.loc[begin_index:end_index]

        # Exclude points that do not have segment context as their context
        res_segments_excluded = overlapping_res_segments[
            overlapping_res_segments['context'] == segment_context]

        # Find size of the longest discovered segment
        longest_segment_size = res_segments_excluded.groupby(['segment_id'])[
            'context'].count().max()

        coverage = longest_segment_size / len(obs_segment)

        total_coverage += coverage

    coverage = total_coverage / total_obs_segments_count

    print(coverage)

    return coverage


def calculate_total_purity():

    total_purity = 0
    processed_purities = 0

    # for i in range(len(all_res_files)):
    for i in range(1):
        # file_path = all_res_files[i]
        file_path = "AIS_trajs/res/219000855-4.csv"

        file_name = os.path.basename(file_path)
        # print(file_name)

        obs_df = read_csv(os.path.join(
            INPUT_OBS_FOLDER, file_name), delimiter=",")
        res_df = read_csv(file_path, delimiter=",")

        # calculate_purity(obs_df, res_df)
        calculate_coverage(obs_df, res_df)

        # obs_df_sailing_1 = obs_df[obs_df['label'] == "01-sailing"]
        # obs_df_fishing_2 = obs_df[obs_df['label'] == "02-fishing"]
        # obs_df_sailing_3 = obs_df[obs_df['label'] == "03-sailing"]

        # first_seg_index = obs_df_sailing_1.iloc[0].name
        # second_seg_index = obs_df_fishing_2.iloc[0].name
        # if len(obs_df_sailing_3) > 0:
        #     third_seg_index = obs_df_sailing_3.iloc[0].name

        # res_df_segmented = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
        #     1)).cumsum())

        # res_df_sailing_1 = res_df_segmented.loc[first_seg_index:second_seg_index]
        # res_df_fishing_2 = res_df_segmented.loc[second_seg_index +
        #                                         1: third_seg_index]
        # if len(obs_df_sailing_3) > 0:
        #     res_df_sailing_3 = res_df_segmented.loc[third_seg_index + 1:]

        # sailing_1 = res_df_sailing_1[res_df_sailing_1['context'] == "SAILING"]
        # fishing_2 = res_df_fishing_2[res_df_fishing_2['context'] == "FISHING"]
        # sailing_3 = res_df_sailing_3[res_df_sailing_3['context'] == "SAILING"]

        # sailing_1_purity = sailing_1.groupby(
        #     ['segment_id'])['context'].count().max()
        # fishing_2_purity = fishing_2.groupby(
        #     ['segment_id'])['context'].count().max()
        # sailing_3_purity = 0
        # if len(obs_df_sailing_3) > 0:
        #     sailing_3_purity = sailing_3.groupby(
        #         ['segment_id'])['context'].count().max() / len(res_df_sailing_3)

        # if (len(res_df_fishing_2) == 0):
        #     pass
        # else:
        #     purity = sailing_1_purity / \
        #         len(res_df_sailing_1) + fishing_2_purity / \
        #         len(res_df_fishing_2) + sailing_3_purity
        #     processed_purities += 1
        # total_purity += purity / 3

    # return total_purity / processed_purities
    return total_purity


def calculate_total_coverage():
    total_coverage = 0
    processed_coverage = 0

    for i in range(len(all_res_files)):
        # for i in range(1):
        file_path = all_res_files[i]

        file_name = os.path.basename(file_path)

        obs_df = read_csv(os.path.join(
            INPUT_OBS_FOLDER, file_name), delimiter=",")
        res_df = read_csv(file_path, delimiter=",")

        obs_df_sailing_1 = obs_df[obs_df['label'] == "01-sailing"]
        obs_df_fishing_2 = obs_df[obs_df['label'] == "02-fishing"]
        obs_df_sailing_3 = obs_df[obs_df['label'] == "03-sailing"]

        first_seg_index = obs_df_sailing_1.iloc[0].name
        second_seg_index = obs_df_fishing_2.iloc[0].name
        if len(obs_df_sailing_3) > 0:
            third_seg_index = obs_df_sailing_3.iloc[0].name

        res_df_segmented = res_df.assign(segment_id=lambda x: (x['context'] != x['context'].shift(
            1)).cumsum())

        res_df_sailing_1 = res_df_segmented.loc[first_seg_index:second_seg_index]
        res_df_fishing_2 = res_df_segmented.loc[second_seg_index +
                                                1: third_seg_index]
        if len(obs_df_sailing_3) > 0:
            res_df_sailing_3 = res_df_segmented.loc[third_seg_index + 1:]

        sailing_1_coverage = res_df_sailing_1.groupby(
            ['segment_id'])['context'].count().max()
        fishing_2_coverage = res_df_fishing_2.groupby(
            ['segment_id'])['context'].count().max()
        sailing_3_coverage = 0
        if len(obs_df_sailing_3) > 0:
            sailing_3_coverage = res_df_sailing_3.groupby(
                ['segment_id'])['context'].count().max() / len(res_df_sailing_3)

        if (len(res_df_fishing_2) == 0):
            pass
        else:
            coverage = sailing_1_coverage / \
                len(res_df_sailing_1) + fishing_2_coverage / \
                len(res_df_fishing_2) + sailing_3_coverage
            processed_coverage += 1
        total_coverage += coverage

    return total_coverage


if __name__ == "__main__":
    print("Total purity is {}".format(calculate_total_purity()))
    # print("Total coverage is {}".format(calculate_total_coverage()))
