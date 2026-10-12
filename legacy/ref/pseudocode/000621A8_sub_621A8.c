/* ea:   0x000621A8 */
/* name: sub_621A8 */
/* size: 116 */
size_t __fastcall sub_621A8(_DWORD *a1, void *a2, size_t a3)
{
  int v6; // r4
  int v7; // r3
  unsigned int v8; // r1

  if ( (*(unsigned __int8 (__fastcall **)(_DWORD *))(*a1 + 36))(a1) == 0 || a2 == nullptr )
    __debugbreak();
  v6 = a1[3];
  v7 = a1[5];
  v8 = a1[4] - v6;
  if ( v7 + a3 > v8 )
    a3 = v8 - v7;
  memcpy(a2, (const void *)(v7 + v6), a3);
  a1[5] += a3;
  return a3;
}

