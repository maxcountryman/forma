select
  t1.name as t1_name,
  t2.name as t2_name,
  t2.kind
from
  t1
  join t2 using (t1_id);
