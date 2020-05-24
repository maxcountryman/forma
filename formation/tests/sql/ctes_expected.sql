with venue_sales as (
    select
      venuename,
      venuecity,
      sum(pricepaid) as venuename_sales
    from
      sales, venue, event
    where
      venue.venueid = event.venueid
      and event.eventid = sales.eventid
    group by
      venuename, venuecity
  ),
  top_venues as (select venuename from venue_sales where venuename_sales > 800000)

select
  venuename,
  venuecity,
  venuestate,
  sum(qtysold) as venue_qty,
  sum(pricepaid) as venue_sales
from
  sales, venue, event
where
  venue.venueid = event.venueid
  and event.eventid = sales.eventid
  and venuename in (select venuename from top_venues)
group by
  venuename, venuecity, venuestate
order by
  venuename;
