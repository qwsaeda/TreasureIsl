/* ea:   0x00039BC4 */
/* name: sub_39BC4 */
/* size: 60 */
_DWORD *__fastcall sub_39BC4(int a1, int *a2)
{
  _DWORD *result; // r0
  _DWORD *v4; // r4
  int v5; // t1

  result = sub_34BEC(a1, 1);
  if ( result != (_DWORD *)-16 )
  {
    v5 = *a2;
    v4 = a2 + 1;
    result[4] = v5;
    result[5] = *v4;
    result[6] = v4[1];
  }
  return result;
}

