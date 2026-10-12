/* ea:   0x0005AF94 */
/* name: sub_5AF94 */
/* size: 88 */
_DWORD *__fastcall sub_5AF94(_DWORD *a1)
{
  _DWORD *v2; // r0
  int v3; // r1

  sub_5AE80((int)a1);
  *a1 = off_7BB68;
  v2 = a1;
  a1[10] = 0;
  v3 = 4;
  do
  {
    --v3;
    v2[292] = 0;
    v2[296] = 9;
    v2[300] = 2;
    v2[268] = 0;
    ++v2;
  }
  while ( v3 != 0 );
  return a1;
}

