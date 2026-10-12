/* ea:   0x00033324 */
/* name: sub_33324 */
/* size: 148 */
int __fastcall sub_33324(_DWORD *a1, char *a2, char *a3)
{
  char *v4; // r6
  int result; // r0
  char *v7; // r4

  v4 = a2;
  if ( a2 == *(char **)(*a1 + 8) && a3 == (char *)*a1 )
    return sub_30CD8(a1);
  while ( 1 )
  {
    result = v4 != a3;
    if ( v4 == a3 )
      break;
    v7 = v4;
    v4 = (char *)sub_16A30(v4);
    sub_30C44(a1, v7);
  }
  return result;
}

