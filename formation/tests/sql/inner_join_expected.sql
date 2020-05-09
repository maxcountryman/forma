select
  catgroup1,
  sold,
  unsold
from
  (
    select
      catgroup,
      sum(qtysold) as sold
    from
      category as c, event as e, sales as s
    where
      c.catid = e.catid
      and e.eventid = s.eventid
    group by
      catgroup
  ) as a (catgroup1, sold)
  join (
    select
      catgroup,
      sum(numtickets) - sum(qtysold) as unsold
    from
      category as c, event as e, sales as s, listing as l
    where
      c.catid = e.catid
      and e.eventid = s.eventid
      and s.listid = l.listid
    group by
      catgroup
  ) as b (catgroup2, unsold) on a.catgroup1 = b.catgroup2
order by
  1