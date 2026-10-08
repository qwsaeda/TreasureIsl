/* ea:   0x00065034 */
/* name: sub_65034 */
/* size: 88 */
size_t __fastcall sub_65034(_DWORD *a1, void *a2, size_t a3)
{
  size_t v5; // r6
  int v6; // r3
  unsigned int v7; // r1

  v5 = a3;
  if ( a2 == nullptr )
    __debugbreak();
  v6 = a1[6];
  v7 = a1[5];
  if ( v6 + a3 > v7 )
    v5 = v7 - v6;
  memcpy(a2, (const void *)(v6 + a1[4]), v5);
  a1[6] += v5;
  return v5;
}

