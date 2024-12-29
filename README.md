# trainer
![CI](https://github.com/gavinmead/trainer/actions/workflows/build.yaml/badge.svg)
[![codecov](https://codecov.io/gh/gavinmead/trainer/graph/badge.svg?token=9GZ3A8MCMF)](https://codecov.io/gh/gavinmead/trainer)

A simple application to track my strength training program

The trainer is built around a very simple training protocol of 5x5 for barbell lifts (Squats, Bench Press
and Deadlift) and a 10x10 for Kettlebell Swings.

The trainer defines a set of training plans that focus on a particular lift across 90%, 80%, 70% 
of the one-rep max and schedules them.  The intention of the app is to make logging the training sessions
as simple as possible and using the data captured to make recommendations on step loading.

### Flow

1. Start a training session
2. Select the prescribed plan or select a particular one.
3. For each set, acknowledge completion of the set for reps and target weight along with a Modified RPE value.  If you did
not complete, specify the number of reps and set the weight (if you decided to change it)
4. Repeat Step 3 until all sets of the training plan are completed.
5. Acknowledge Training Session is complete.
6. Repeat 1-5 until strong.

### Definitions

Training Plan:  A set of training plan entries
Training Plan Entry:  An exercise, a projected set of reps, a set value and projected weight
Training Plan Snapshot:  A copy of the training plan linked to a given training session. The reason for this is to prevent changes
to the training plan from cascading to previous training sessions.
Training Log Entry: A training log entry contains the actual work done compared to its associated training plan entry
Training Log:  A set of training log entries that is associated with a training session that is linked to a training plan via a snapshot
Training Session: Captures global details of a training session (date, time, duration) and contains a training log.
Training Sequencer: A set of training plans that is used to prescribe the next training session.

#### Sequencer Example

My training goal is consistency as I want to train at least 6 days per week.  In order to make this sustainable I
should aim to do no more than 90% of each lift twice per week and do it in a wave of 70, 80, 90, 70, 80, 90 varied by 
lift.  

For example:
* Training Plan: "9DL/8SQ/7BP" would 90% deadlifts, 80% for squats, 70% for bench in that order.
* Training Plan: "9SQ/8BP/7DL" would be 90% squats, 80% for bench, 70% for deadlifts in that order.
* Training Plan: "9BP/8DL/7SQ" would be 90% bench, 80% for deadlifts, 70% for squats in that order

A Training Sequence would be:
* "9DL/8SQ/7BP
* "9SQ/8BP/7DL"
* "9BP/8DL/7SQ"

Repeated

#### Step Loader

Ideally, as the RPE goes down across training session, that is a signal that one is getting stronger and it is time to introduce more weight.
The Step Loader will modify the training plan to introduce a weight increment across all 3 plans.  The step-loader will do the following:
* Increment the weight by some value (lift dependent)
* Update Set 3 to use this new weight
* Update Set 4 to use this new weight after 2-4 weeks after Set 3
* Update Set 2 to use this new weight after 2-4 weeks after Set 4
* Update Set 5 to use this new weight after 2-4 weeks after Set 2
* Update Set 1 to use this new weight after 2-4 weeks after Set 5
* Stay at this weight for at least 4 weeks

Note that step-loading is an infrequent occurrence over the course of a year.

### Structure

This application is a collection of components including:
* a core api library
* an axum based REST Server that wraps the core API (hexagonal architecture)
* a hyper based REST Client
* a sqlite based backend that is replicated via litestream to S3
* a slint based front end for a low cost touchscreen that can be used in the training space
* a slint based MacOS Application for defining training plans and viewing results
* CDK setup for an AWS API Gateway, Lambda with axum and other stuff to host the api.  Uses mTLS
 
