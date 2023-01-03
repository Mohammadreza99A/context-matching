# Context Matching for fishing trajectory

# History

## Simple version

Lots of error (success rate of ~ 70%) because in the update part of the particle filtering we are using the position of the sample in order to calculate the new position of the sample. Something like:
```rust
let new_pos = Point {
    x: sample.pos.x + (new_speed * time_diff * new_dir.x),
    y: sample.pos.y + (new_speed * time_diff * new_dir.y),
};
```

## Improved update process

To improve the accuracy, we use the observation position instead of sample position to calculate the new position of the sample.
```rust
let new_pos = Point {
    x: observation.pos.x + (new_speed * time_diff * new_dir.x),
    y: observation.pos.y + (new_speed * time_diff * new_dir.y),
};
```
Success rate increases to ~ 80% but there are still a lot of portions of the result that need to be smoothed. 

## Smoothing the result with sliding window

To improve the result, we have to change the context of the points of the same context that are small portions surrounded by two big portions of the opposite context. To do this, we use a sliding window technique which does the correction between importance sampling and resampling process. This is done during the processing and not as a pros processing job because of how big the data would be if we wanted to do it in post processing.

This method is a numerical method and although it has very good success rate (~ 90%), we must find a way that this could be done based on context and not numerical values.

Here is how we apply this method. We define a vector of float tuples with a fixed size. Tuples in this vector are the number of sailing and fishing context for a set of samples for a given result. E.g. imagine that we have processed 5 observations until now and for each observation we have 100 samples. Between these 100 samples for each processed observation, we have a number of samples which have sailing context and a number of samples which have fishing context. The sum of these two numbers is 100 of course. So our vector should look something like this
$$[(25,75), (50,50), (60,40), (75,25), (85,15)]$$

In the particle filtering process, after updating samples and applying the importance sampling process, we add the sample with max weight to the results and apply the resampling process. This smoothing happens right before applying the resampling process. We change the context of the middle element of the window based on the elements which are around it. We count the number of sailing and fishing contexts around the middle element of the window. If number of sailing context if bigger than the number of fishing context and that the middle element has a good sailing context (> 50), we change the context to sailing for this element (also in result set). We do the same thing for the fishing context. Here is a small code for this

```rust
if sailing_total > fishing_total {
    if self.context_smoothing_window[mid_index].0 >= 50 { // sailing of mid elem
        return ContextType::SAILING;
    }
} else {
    if self.context_smoothing_window[mid_index].1 >= 50 { // fishing of mid elem
        return ContextType::FISHING;
    }
}
```

## Eliminating error for points near the shore

Basically, what we want to do is to penalize the weight of samples that have fishing context and are near to the shore without introducing some kind of a threshold. 

To achieve this, we could consider a weight scaling function $f(d)$ that only depends on the distance from a line (line being the shore here). We already have this distance for each observation (`distanceToShore`) as input data. So we could use that. And the value of this $f(d)$ function will be multiplied to the calculated weight in importance sampling step if the context of the sample is fishing.

$f(d)$ should be a non-decreasing function for $d \geq 0$ with $lim_{d \rightarrow \infty} f(d) = 1$ with $f(0) = \epsilon$ where $\epsilon$ is the minimum weight scaling at exactly on the line. E.g. if $\epsilon = 0$, the weight on the line is $0$.

Hence, we can use an exponential function for this with the form of
$$f(d) = 1 - Ce^{-d/d_0}$$

where $1-C$ is the minimum weight scale at $d=0$, i.e. $C = 1 - \epsilon$ and $d_0$ defines the range. At $d = d_0$, the weight scale is $1 - Ce^{-1} = 1 - 0.368C$.

Here the range ($d_0$) is important because it defines how the penalization works for points. For now, we have `4000 + obs[0].distanceToShore` but we have to see how to find a more generic value instead of $4000$ which is hard coded.

## Thoughts on some kind of multi layered context matching 

What we need is to try and reapply the particle filtering on the obtained result. So now we have to figure out what we need to keep from the first layer as data and how to use it in second layer so that we can improve the result.

- Could we exploit the idea of Kalman filters in second layer?
- How about the idea of Viterbi algorithm? (even of possible, it would be long because it has to be applied for each set of samples)