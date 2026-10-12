/* ea:   0x00033FA0 */
/* name: sub_33FA0 */
/* size: 140 */
int __fastcall sub_33FA0(_DWORD *a1)
{
  _DWORD *v2; // r2
  int result; // r0

  v2 = a1 + 7;
  if ( a1[7] == a1[8] )
  {
    result = a1[12];
    if ( result != 0 )
    {
      a1[12] = result - 1;
    }
    else
    {
      result = a1[5];
      a1[12] = ((result - a1[4]) >> 2) - 1;
    }
  }
  else
  {
    result = a1[13];
    if ( result != 0 )
    {
      a1[13] = result - 1;
    }
    else
    {
      result = v2[1];
      a1[13] = ((result - *v2) >> 2) - 1;
    }
  }
  return result;
}

