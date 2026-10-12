/* ea:   0x0003129C */
/* name: sub_3129C */
/* size: 56 */
int __fastcall sub_3129C(_BYTE *a1)
{
  int result; // r0

  result = (unsigned __int8)a1[50];
  if ( (_BYTE)result != 0 )
  {
    result = (unsigned __int8)a1[41];
    if ( a1[41] != 0 )
    {
      result += 255;
      a1[41] = result;
      if ( (_BYTE)result == 0 )
      {
        a1[40] = 1;
        return 1;
      }
    }
  }
  return result;
}

