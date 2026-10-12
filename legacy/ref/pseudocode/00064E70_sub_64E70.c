/* ea:   0x00064E70 */
/* name: sub_64E70 */
/* size: 72 */
int __fastcall sub_64E70(int a1, int a2, int a3)
{
  if ( a2 != a3 )
  {
    if ( a2 == 8 )
    {
      if ( a3 == 4 )
        return 0;
    }
    else if ( a2 == 24 && (a3 == 4 || a3 == 8) )
    {
      return 0;
    }
  }
  return 1;
}

