select
  salesid,
  listid,
  sum(pricepaid)
from
  sales as s
where
  qtysold = (select max(numtickets) from listing as l where s.listid = l.listid)
group by
  1, 2
order by
  1, 2
limit
  5;
