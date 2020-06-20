select
  employee_id, first_name, last_name, salary
from
  employees
order by
  salary desc
offset 0 rows
fetch first 1 rows only;
select
  employee_id, first_name, last_name, salary
from
  employees
order by
  salary desc
offset 5 rows
fetch first 10 percent rows with ties;
select
  employee_id, first_name, last_name, salary
from
  employees
order by
  salary desc
offset 1 row
fetch first rows only;
