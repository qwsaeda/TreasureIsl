/* ea:   0x0004A980 */
/* name: sub_4A980 */
/* size: 100 */
int __fastcall sub_4A980(_DWORD *a1, unsigned int *a2)
{
  unsigned __int8 v4; // r3
  unsigned int v5; // r0
  unsigned int v6; // r1
  int v7; // r2
  bool v8; // zf
  int result; // r0

  v4 = 0;
  *a2 = 0;
  v5 = (unsigned int)(*(unsigned __int8 *)(a1[3] + 22) << 26) >> 28;
  v6 = v5 + 1;
  if ( v5 == 0 )
  {
    v6 = a1[2];
    v7 = *(unsigned __int8 *)(a1[5] + 50);
    v8 = v7 == v6;
    if ( v7 != v6 )
      v8 = v7 == 0;
    if ( !v8 )
      v4 = 1;
  }
  if ( v4 != 0 )
    v6 = *(unsigned __int8 *)(a1[5] + 50);
  result = v4;
  if ( v4 != 0 )
    *a2 = v6;
  return result;
}

