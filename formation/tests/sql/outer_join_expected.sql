select
  listing.listid,
  sum(pricepaid) as price,
  sum(commission) as comm
from
  listing
  left join sales on sales.listid = listing.listid
where
  listing.listid between 1 and 5
group by
  1
order by
  1