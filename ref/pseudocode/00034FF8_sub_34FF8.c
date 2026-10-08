/* ea:   0x00034FF8 */
/* name: sub_34FF8 */
/* size: 156 */
BOOL __fastcall sub_34FF8(int a1, int a2)
{
  int v4; // r7
  int v5; // r0
  int v6; // r5
  int v7; // r4
  int v8; // r0

  v4 = (*(int (__fastcall **)(int))(*(_DWORD *)a1 + 24))(a1);
  v5 = (*(int (__fastcall **)(int))(*(_DWORD *)a1 + 28))(a1);
  if ( v5 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(v5 + 8);
  v7 = (*(int (__fastcall **)(int))(*(_DWORD *)a2 + 24))(a2);
  v8 = (*(int (__fastcall **)(int))(*(_DWORD *)a2 + 28))(a2);
  if ( v8 == 0 )
    __debugbreak();
  return v6 + v4 < (unsigned int)(*(_DWORD *)(v8 + 8) + v7);
}

