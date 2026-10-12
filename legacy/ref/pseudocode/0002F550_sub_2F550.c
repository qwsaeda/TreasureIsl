/* ea:   0x0002F550 */
/* name: sub_2F550 */
/* size: 160 */
int __fastcall sub_2F550(_DWORD *a1, _DWORD *a2)
{
  int v4; // r6
  _BYTE v6[8]; // [sp+0h] [bp-24h] BYREF
  _DWORD v7[7]; // [sp+8h] [bp-1Ch] BYREF

  v4 = sub_343E4();
  if ( v4 != *a1 && *a2 >= *(_DWORD *)(v4 + 16) )
    return v4 + 20;
  v7[0] = *a2;
  memcpy(&v7[1], v6, 6u);
  return *(_DWORD *)sub_35A58(a1, v6, v4, v7) + 20;
}

