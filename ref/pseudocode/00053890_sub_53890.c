/* ea:   0x00053890 */
/* name: sub_53890 */
/* size: 176 */
int __fastcall sub_53890(int *a1, unsigned __int16 a2)
{
  int v2; // r8
  int v3; // r5
  int v4; // r6
  int v5; // r4

  v2 = 0;
  if ( *a1 <= 0 )
    return 0;
  v3 = a1[1];
  while ( 1 )
  {
    v4 = 0;
    if ( v3 > 0 )
      break;
LABEL_8:
    if ( ++v2 >= *a1 )
      return 0;
  }
  v5 = *(_DWORD *)(a1[2] + 4 * v2);
  while ( (*(_BYTE *)(*(_DWORD *)v5 + 22) & 0x3F) != 5
       || a2 - 1 != (unsigned int)(*(unsigned __int16 *)(*(_DWORD *)v5 + 24) << 24) >> 30 )
  {
    ++v4;
    v5 += 4;
    if ( v4 >= v3 )
      goto LABEL_8;
  }
  return *(_DWORD *)(*(_DWORD *)(a1[2] + 4 * v2) + 4 * v4);
}

