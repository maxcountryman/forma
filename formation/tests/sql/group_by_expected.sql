select
  listid,
  eventid,
  sum(pricepaid) as revenue,
  count(qtysold) as numtix
from
  sales
group by
  1, 2
order by
  3, 4, 2, 1
limit
  5;
