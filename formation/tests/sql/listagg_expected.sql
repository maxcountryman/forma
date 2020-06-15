select
  listagg(distinct sellerid, ',' on overflow truncate with count)
  within group (order by dateid) as sellers,
  listagg(dateid, ',' on overflow error)
  within group (order by dateid) as dates,
  listagg(customerid, ',' on overflow truncate '%' without count) as customers
from
  winsales;
