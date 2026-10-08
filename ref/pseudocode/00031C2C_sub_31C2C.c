/* ea:   0x00031C2C */
/* name: sub_31C2C */
/* size: 200 */
BOOL __fastcall sub_31C2C(int a1)
{
  unsigned __int16 *v1; // r6

  v1 = (unsigned __int16 *)(a1 + 136);
  return (unsigned int)(*(_DWORD *)(*(_DWORD *)(a1 + 16) + 8) + sub_16C9C((unsigned __int16 *)(a1 + 136), 2)) >= *(_DWORD *)&algn_6D2AC[4 * dword_7FB6C] >> 1
      && (sub_16C9C(v1, 0) > 3 * (unsigned int)word_6D2BA[dword_7FB6C]
       || sub_16C9C(v1, 1) > 2 * (unsigned int)word_6D2BA[dword_7FB6C]);
}

