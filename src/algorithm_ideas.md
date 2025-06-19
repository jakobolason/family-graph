
# Overview of when edges are introduced
When now is a node who is a relative, certain scenarios can happen. 
$$n=last-now$$

| n  | last | now | action                             |
|----|------|-----|------------------------------------|
| -1 | -1   | 0   | no edge, start case                |
| 0  | 0    | 0   | no edge, siblings                  |
| -1 | 0    | 1   | 1 edge, `parent=last; crnt=new`    |
| -2 | 0    | 2   | shouldn't be possible              |
| 1  | 1    | 0   | no edge                            |
| 0  | 1    | 1   | no edge, siblings                  |
| -1 | 1    | 2   | 1 edge, `parent=last; crnt=new`    |
| 2  | 2    | 0   | no edge                            |
| 1  | 2    | 1   | no edge, find last 1. gen relative |
| 0  | 2    | 2   | no edge, sibling                   |
