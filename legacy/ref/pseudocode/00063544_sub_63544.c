/* ea:   0x00063544 */
/* name: sub_63544 */
/* size: 60 */
int __fastcall sub_63544(_DWORD *a1)
{
  int v2; // r0
  bool v3; // nf
  int result; // r0

  v2 = a1[9];
  v3 = v2 < 0;
  if ( v2 <= 0 )
  {
    result = 2;
    if ( v3 )
      result = 3;
    a1[8] = result;
  }
  else if ( (int)a1[10] <= 0 )
  {
    a1[8] = 1;
    return 1;
  }
  else
  {
    a1[8] = 0;
    return 0;
  }
  return result;
}

