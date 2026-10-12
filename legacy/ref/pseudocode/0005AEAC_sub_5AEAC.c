/* ea:   0x0005AEAC */
/* name: sub_5AEAC */
/* size: 44 */
unsigned int __fastcall sub_5AEAC(_DWORD *a1, unsigned int a2)
{
  int v3; // r2
  _DWORD *v4; // r0
  unsigned int result; // r0

  v3 = a2;
  v4 = (_DWORD *)a1[1];
  if ( a2 > 0x40 )
    v3 = 64;
  a1[2] = v3;
  result = (unsigned int)(*v4 * v3) >> 6;
  a1[3] = result;
  return result;
}

