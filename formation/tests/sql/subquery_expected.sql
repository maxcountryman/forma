select
  qtr,
  sum(pricepaid) as qtrsales,
  (
    select
      sum(pricepaid)
    from
      sales
      join _date on sales.dateid = _date.dateid
    where
      qtr = '1'
      and year = 2008
  ) as q1sales
from
  sales
  join _date on sales.dateid = _date.dateid
where
  qtr in ('2', '3')
  and year = 2008
group by
  qtr
order by
  qtr;
