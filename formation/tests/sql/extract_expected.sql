select salesid, extract(day from saletime) as weeknum from sales where pricepaid > 9999 order by 2;
