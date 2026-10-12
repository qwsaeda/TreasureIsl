/* ea:   0x00045E50 */
/* name: sub_45E50 */
/* size: 40 */
void __fastcall sub_45E50(_DWORD *a1)
{
  DWORD v2; // r1

  v2 = GetTickCount() - *a1;
  a1[1] = v2;
  if ( v2 < 0x29 )
    Sleep(41 - v2);
}

