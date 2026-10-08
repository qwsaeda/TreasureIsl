/* ea:   0x0006A4F8 */
/* name: sub_6A4F8 */
/* size: 148 */
_DWORD *__fastcall sub_6A4F8(_DWORD *a1)
{
  int v2; // r0
  int v3; // r1
  char v4; // r3
  _DWORD *v5; // r0
  _DWORD *v6; // r4
  _DWORD v8[8]; // [sp+4h] [bp-20h] BYREF

  if ( a1 == nullptr || a1[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v2 = a1[1];
  v3 = a1[2];
  v4 = *(_BYTE *)a1;
  v8[0] = 0;
  v8[1] = v2;
  v8[2] = 0;
  v8[3] = v3;
  v5 = sub_62694(v2, v3, 16, v4);
  v6 = v5;
  if ( v5 == nullptr )
    return nullptr;
  sub_5CA88(v5, (int)v8, a1);
  return v6;
}

