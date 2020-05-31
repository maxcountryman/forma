select
  salesid, sellerid, qty, rank() over(partition by sellerid order by qty desc) as rank
from
  winsales
order by
  2, 3, 1;
