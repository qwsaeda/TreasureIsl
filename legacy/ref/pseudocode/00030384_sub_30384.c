/* ea:   0x00030384 */
/* name: sub_30384 */
/* size: 52 */
int __fastcall sub_30384(int *a1)
{
  int result; // r0
  __int16 v3; // r1

  result = *a1;
  if ( result != 0 )
  {
    v3 = *((_WORD *)a1 + 2) - 1;
    result = v3;
    *((_WORD *)a1 + 2) = v3;
    if ( v3 <= 0 )
    {
      *a1 = 0;
      return 0;
    }
  }
  return result;
}

